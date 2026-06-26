package com.rve.systemmonitor.domain.model

import kotlinx.serialization.Serializable

@Serializable
enum class OverlayPosition {
    FREE,
    TOP_LEFT,
    TOP_CENTER,
    TOP_RIGHT,
}
