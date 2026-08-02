package com.rve.systemmonitor.utils

import android.content.Context
import android.graphics.Color
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.floatPreferencesKey
import androidx.datastore.preferences.core.intPreferencesKey
import androidx.datastore.preferences.core.longPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.core.stringSetPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import com.rve.systemmonitor.domain.model.OverlayPosition
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

val Context.overlayDataStore: DataStore<Preferences> by preferencesDataStore(name = "overlay_settings")

class OverlayPreferences(private val context: Context) {
    companion object {
        val IS_OVERLAY_ENABLED_KEY = booleanPreferencesKey("is_overlay_enabled")
        val IS_FPS_ENABLED_KEY = booleanPreferencesKey("is_fps_enabled")
        val IS_RAM_ENABLED_KEY = booleanPreferencesKey("is_ram_enabled")
        val IS_RAM_PERCENTAGE_ENABLED_KEY = booleanPreferencesKey("is_ram_percentage_enabled")
        val IS_RAM_GB_ENABLED_KEY = booleanPreferencesKey("is_ram_gb_enabled")
        val IS_BATTERY_TEMP_ENABLED_KEY = booleanPreferencesKey("is_battery_temp_enabled")
        val IS_CPU_TEMP_ENABLED_KEY = booleanPreferencesKey("is_cpu_temp_enabled")
        val OVERLAY_UPDATE_INTERVAL_KEY = longPreferencesKey("overlay_update_interval")
        val OVERLAY_TEXT_SIZE_KEY = floatPreferencesKey("overlay_text_size")
        val OVERLAY_BG_OPACITY_KEY = floatPreferencesKey("overlay_bg_opacity")
        val OVERLAY_PADDING_KEY = intPreferencesKey("overlay_padding")
        val OVERLAY_TEXT_COLOR_KEY = intPreferencesKey("overlay_text_color")
        val IS_VERTICAL_LAYOUT_KEY = booleanPreferencesKey("is_vertical_layout")
        val OVERLAY_CORNER_RADIUS_KEY = intPreferencesKey("overlay_corner_radius")
        val OVERLAY_POSITION_KEY = stringPreferencesKey("overlay_position")
        val OVERLAY_X_KEY = intPreferencesKey("overlay_x")
        val OVERLAY_Y_KEY = intPreferencesKey("overlay_y")
        val IS_AUTO_TOGGLE_ENABLED_KEY = booleanPreferencesKey("is_auto_toggle_enabled")
        val AUTO_TOGGLE_APPS_KEY = stringSetPreferencesKey("auto_toggle_apps")
    }

    val isOverlayEnabledFlow: Flow<Boolean> = context.overlayDataStore.getValueFlow(IS_OVERLAY_ENABLED_KEY, false)
    val isFpsEnabledFlow: Flow<Boolean> = context.overlayDataStore.getValueFlow(IS_FPS_ENABLED_KEY, false)
    val isRamEnabledFlow: Flow<Boolean> = context.overlayDataStore.getValueFlow(IS_RAM_ENABLED_KEY, false)
    val isRamPercentageEnabledFlow: Flow<Boolean> = context.overlayDataStore.getValueFlow(IS_RAM_PERCENTAGE_ENABLED_KEY, false)
    val isRamGbEnabledFlow: Flow<Boolean> = context.overlayDataStore.getValueFlow(IS_RAM_GB_ENABLED_KEY, false)
    val isBatteryTempEnabledFlow: Flow<Boolean> = context.overlayDataStore.getValueFlow(IS_BATTERY_TEMP_ENABLED_KEY, false)
    val isCpuTempEnabledFlow: Flow<Boolean> = context.overlayDataStore.getValueFlow(IS_CPU_TEMP_ENABLED_KEY, false)
    val overlayUpdateIntervalFlow: Flow<Long> = context.overlayDataStore.getValueFlow(OVERLAY_UPDATE_INTERVAL_KEY, 1000L)
    val overlayTextSizeFlow: Flow<Float> = context.overlayDataStore.getValueFlow(OVERLAY_TEXT_SIZE_KEY, 14f)
    val overlayBgOpacityFlow: Flow<Float> = context.overlayDataStore.getValueFlow(OVERLAY_BG_OPACITY_KEY, 0.5f)
    val overlayPaddingFlow: Flow<Int> = context.overlayDataStore.getValueFlow(OVERLAY_PADDING_KEY, 16)
    val overlayTextColorFlow: Flow<Int> = context.overlayDataStore.getValueFlow(OVERLAY_TEXT_COLOR_KEY, Color.GREEN)
    val isVerticalLayoutFlow: Flow<Boolean> = context.overlayDataStore.getValueFlow(IS_VERTICAL_LAYOUT_KEY, false)
    val overlayCornerRadiusFlow: Flow<Int> = context.overlayDataStore.getValueFlow(OVERLAY_CORNER_RADIUS_KEY, 8)
    val overlayPositionFlow: Flow<OverlayPosition> = context.overlayDataStore.data.map { preferences ->
        try {
            OverlayPosition.valueOf(preferences[OVERLAY_POSITION_KEY] ?: OverlayPosition.FREE.name)
        } catch (e: IllegalArgumentException) {
            OverlayPosition.FREE
        }
    }
    val overlayXFlow: Flow<Int> = context.overlayDataStore.getValueFlow(OVERLAY_X_KEY, 100)
    val overlayYFlow: Flow<Int> = context.overlayDataStore.getValueFlow(OVERLAY_Y_KEY, 100)
    val isAutoToggleEnabledFlow: Flow<Boolean> = context.overlayDataStore.getValueFlow(IS_AUTO_TOGGLE_ENABLED_KEY, false)
    val autoToggleAppsFlow: Flow<Set<String>> = context.overlayDataStore.getValueFlow(AUTO_TOGGLE_APPS_KEY, emptySet())

    suspend fun saveIsOverlayEnabled(enabled: Boolean) = context.overlayDataStore.setValue(IS_OVERLAY_ENABLED_KEY, enabled)
    suspend fun saveIsFpsEnabled(enabled: Boolean) = context.overlayDataStore.setValue(IS_FPS_ENABLED_KEY, enabled)
    suspend fun saveIsRamEnabled(enabled: Boolean) = context.overlayDataStore.setValue(IS_RAM_ENABLED_KEY, enabled)
    suspend fun saveIsRamPercentageEnabled(enabled: Boolean) = context.overlayDataStore.setValue(IS_RAM_PERCENTAGE_ENABLED_KEY, enabled)
    suspend fun saveIsRamGbEnabled(enabled: Boolean) = context.overlayDataStore.setValue(IS_RAM_GB_ENABLED_KEY, enabled)
    suspend fun saveIsBatteryTempEnabled(enabled: Boolean) = context.overlayDataStore.setValue(IS_BATTERY_TEMP_ENABLED_KEY, enabled)
    suspend fun saveIsCpuTempEnabled(enabled: Boolean) = context.overlayDataStore.setValue(IS_CPU_TEMP_ENABLED_KEY, enabled)
    suspend fun saveOverlayUpdateInterval(delayMillis: Long) = context.overlayDataStore.setValue(OVERLAY_UPDATE_INTERVAL_KEY, delayMillis)
    suspend fun saveOverlayTextSize(size: Float) = context.overlayDataStore.setValue(OVERLAY_TEXT_SIZE_KEY, size)
    suspend fun saveOverlayBgOpacity(opacity: Float) = context.overlayDataStore.setValue(OVERLAY_BG_OPACITY_KEY, opacity)
    suspend fun saveOverlayPadding(padding: Int) = context.overlayDataStore.setValue(OVERLAY_PADDING_KEY, padding)
    suspend fun saveOverlayTextColor(color: Int) = context.overlayDataStore.setValue(OVERLAY_TEXT_COLOR_KEY, color)
    suspend fun saveIsVerticalLayout(vertical: Boolean) = context.overlayDataStore.setValue(IS_VERTICAL_LAYOUT_KEY, vertical)
    suspend fun saveOverlayCornerRadius(radius: Int) = context.overlayDataStore.setValue(OVERLAY_CORNER_RADIUS_KEY, radius)
    suspend fun saveOverlayPosition(position: OverlayPosition) = context.overlayDataStore.setValue(OVERLAY_POSITION_KEY, position.name)
    suspend fun saveOverlayX(x: Int) = context.overlayDataStore.setValue(OVERLAY_X_KEY, x)
    suspend fun saveOverlayY(y: Int) = context.overlayDataStore.setValue(OVERLAY_Y_KEY, y)
    suspend fun saveIsAutoToggleEnabled(enabled: Boolean) = context.overlayDataStore.setValue(IS_AUTO_TOGGLE_ENABLED_KEY, enabled)
    suspend fun saveAutoToggleApps(apps: Set<String>) = context.overlayDataStore.setValue(AUTO_TOGGLE_APPS_KEY, apps)
}
