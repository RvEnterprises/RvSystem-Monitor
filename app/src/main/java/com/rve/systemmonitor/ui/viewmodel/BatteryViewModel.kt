package com.rve.systemmonitor.ui.viewmodel

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rve.systemmonitor.domain.model.Battery
import com.rve.systemmonitor.domain.model.BatteryDataPoint
import com.rve.systemmonitor.domain.repository.BatteryRepository
import com.rve.systemmonitor.domain.repository.SettingsRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import javax.inject.Inject
import kotlinx.collections.immutable.ImmutableList
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

@OptIn(FlowPreview::class)
@HiltViewModel
class BatteryViewModel @Inject constructor(private val batteryRepository: BatteryRepository, settingsRepository: SettingsRepository) :
    ViewModel() {
    private val batteryStatic = flow {
        emit(batteryRepository.getBatteryInfo())
    }.flowOn(Dispatchers.IO)
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = Battery(),
        )

    private val batteryStream = batteryRepository.getBatteryStream()
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = Battery(),
        )

    val batteryInfo: StateFlow<Battery> = combine(
        batteryStream,
        batteryStatic,
    ) { stream, static ->
        if (static.capacity == 0.0) return@combine stream

        stream.copy(
            health = static.health,
            technology = static.technology,
            capacity = static.capacity,
            cycleCount = static.cycleCount,
            deepSleep = static.deepSleep,
        )
    }.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = Battery(),
    )
    val graphHistorySeconds: StateFlow<Int> = settingsRepository.batteryGraphHistorySeconds
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = 60,
        )

    private val _historyList = mutableListOf<BatteryDataPoint>()

    val batteryHistory: StateFlow<ImmutableList<BatteryDataPoint>> = batteryInfo
        .map { info ->
            _historyList.add(BatteryDataPoint(info.current, info.status))
            val maxHistory = graphHistorySeconds.value
            if (_historyList.size > maxHistory) {
                _historyList.subList(0, _historyList.size - maxHistory).clear()
            }
            _historyList.toImmutableList()
        }
        .combine(graphHistorySeconds) { history: ImmutableList<BatteryDataPoint>, maxHistory: Int ->
            if (history.size > maxHistory) history.takeLast(maxHistory).toImmutableList() else history
        }
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = _historyList.toImmutableList(),
        )

    var hasAnimated by mutableStateOf(false)
        private set

    fun markAsAnimated() {
        hasAnimated = true
    }
}
