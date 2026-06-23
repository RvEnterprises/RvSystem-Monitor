package com.rve.systemmonitor.utils

import com.rve.systemmonitor.shizuku.ShizukuManager
import javax.inject.Inject
import javax.inject.Singleton
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn

@OptIn(kotlinx.coroutines.ExperimentalCoroutinesApi::class)
@Singleton
class FpsMonitor @Inject constructor(
    private val shizukuManager: ShizukuManager,
    private val settingsRepository: com.rve.systemmonitor.domain.repository.SettingsRepository,
) {
    val framesPerSecond: Flow<Int> = combine(
        settingsRepository.useShizuku,
        shizukuManager.isShizukuAvailable,
        shizukuManager.hasPermission
    ) { use, avail, perm ->
        use && avail && perm
    }.flatMapLatest { shizukuReady ->
        flow {
            var initialized = false
            var lastTotalFrames = -1L
            emit(0)

            while (true) {
                if (shizukuReady) {
                    try {
                        if (!initialized) {
                            shizukuManager.executeCommand(
                                "dumpsys SurfaceFlinger --timestats -enable",
                            )
                            initialized = true
                        } else {
                            val output = shizukuManager.executeCommand(
                                "dumpsys SurfaceFlinger --timestats -dump",
                            )

                            val currentTotalFrames = Regex("totalFrames\\s*=\\s*([0-9]+)")
                                .find(output)
                                ?.groupValues?.get(1)
                                ?.toLongOrNull()
                                ?: -1L

                            if (currentTotalFrames != -1L) {
                                if (lastTotalFrames != -1L) {
                                    val fps = (currentTotalFrames - lastTotalFrames).toInt()
                                    emit(fps.coerceAtLeast(0))
                                }
                                lastTotalFrames = currentTotalFrames
                            } else {
                                emit(0)
                            }
                        }
                    } catch (e: Exception) {
                        initialized = false
                        emit(0)
                    }
                } else {
                    emit(0)
                }
                delay(1000)
            }
        }
    }.flowOn(Dispatchers.IO)
}
