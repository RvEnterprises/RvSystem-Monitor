package com.rve.systemmonitor.domain.model

import androidx.compose.runtime.Immutable
import com.rve.systemmonitor.R

@Immutable
data class OS(
    val name: String = "Android",
    val version: String = "unknown",
    val sdk: Int = 0,
    val dessertName: String = "unknown",
    val dessertNameRes: Int = R.string.value_unknown,
    val securityPatch: String = "unknown",
    val hyperOSVersion: String? = null,
)
