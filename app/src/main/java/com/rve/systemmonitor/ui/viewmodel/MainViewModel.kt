package com.rve.systemmonitor.ui.viewmodel

import androidx.compose.runtime.Immutable
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rve.systemmonitor.domain.repository.SettingsRepository
import com.rve.systemmonitor.utils.ThemeMode
import com.rve.systemmonitor.utils.VibrationIntensity
import dagger.hilt.android.lifecycle.HiltViewModel
import javax.inject.Inject
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn

@HiltViewModel
class MainViewModel @Inject constructor(settingsRepository: SettingsRepository) : ViewModel() {
    val uiState: StateFlow<MainUiState> = combine<Any, MainUiState>(
        settingsRepository.themeMode,
        settingsRepository.amoledMode,
        settingsRepository.isSetupCompleted,
        settingsRepository.hapticFeedbackEnabled,
        settingsRepository.vibrationIntensity,
        settingsRepository.autoUpdateEnabled,
        settingsRepository.blurEffectEnabled,
    ) { args ->
        MainUiState.Success(
            themeMode = args[0] as ThemeMode,
            amoledMode = args[1] as Boolean,
            isSetupCompleted = args[2] as Boolean,
            hapticFeedbackEnabled = args[3] as Boolean,
            vibrationIntensity = args[4] as VibrationIntensity,
            autoUpdateEnabled = args[5] as Boolean,
            blurEffectEnabled = args[6] as Boolean,
        )
    }.stateIn(
        scope = viewModelScope,
        initialValue = MainUiState.Loading,
        started = SharingStarted.WhileSubscribed(5_000),
    )
}

sealed interface MainUiState {
    data object Loading : MainUiState

    @Immutable
    data class Success(
        val themeMode: ThemeMode,
        val amoledMode: Boolean,
        val isSetupCompleted: Boolean,
        val hapticFeedbackEnabled: Boolean,
        val vibrationIntensity: VibrationIntensity,
        val autoUpdateEnabled: Boolean,
        val blurEffectEnabled: Boolean,
    ) : MainUiState
}
