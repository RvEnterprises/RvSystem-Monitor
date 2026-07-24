package com.rve.systemmonitor.domain.model

import androidx.compose.runtime.Immutable
import kotlinx.collections.immutable.ImmutableList
import kotlinx.collections.immutable.persistentListOf

@Immutable
data class GPU(
    val renderer: String = "unknown",
    val vendor: String = "unknown",
    val glesVersion: String = "unknown",
    val detailedGlesVersion: String = "unknown",
    val vulkanVersion: String = "unknown",
    val vulkanDriverVersion: String = "unknown",
    val temperature: Double = 0.0,
    val maxTextureSize: Int = 0,
    val extensionsCount: Int = 0,
    val vulkanExtensionsCount: Int = 0,
    val vulkanExtensions: ImmutableList<String> = persistentListOf(),
    val deviceType: String = "unknown",
    val shadingLanguageVersion: String = "unknown",
    val totalMemoryMb: Long = 0,
)
