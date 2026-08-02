package com.rve.systemmonitor.service

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.app.usage.UsageEvents
import android.app.usage.UsageStatsManager
import android.content.Context
import android.content.Intent
import android.graphics.Color
import android.graphics.PixelFormat
import android.graphics.drawable.GradientDrawable
import android.os.IBinder
import android.view.Gravity
import android.view.MotionEvent
import android.view.View
import android.view.WindowManager
import android.widget.TextView
import androidx.core.app.NotificationCompat
import com.rve.systemmonitor.R
import com.rve.systemmonitor.domain.model.OverlayPosition
import com.rve.systemmonitor.domain.repository.OverlayRepository
import com.rve.systemmonitor.utils.BatteryUtils
import com.rve.systemmonitor.utils.CpuUtils
import com.rve.systemmonitor.utils.FpsMonitor
import com.rve.systemmonitor.utils.MemoryUtils
import dagger.hilt.android.AndroidEntryPoint
import javax.inject.Inject
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

@OptIn(ExperimentalCoroutinesApi::class)
@AndroidEntryPoint
class SystemOverlayService : Service() {

    @Inject
    lateinit var overlayRepository: OverlayRepository

    @Inject
    lateinit var fpsMonitor: FpsMonitor

    private var windowManager: WindowManager? = null
    private var overlayView: View? = null
    private var metricsView: MetricsSurfaceView? = null

    private val serviceJob = SupervisorJob()
    private val serviceScope = CoroutineScope(Dispatchers.Main + serviceJob)

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        isRunning = true
        startForeground(NOTIFICATION_ID, createNotification())
        showOverlay()
        startDataPipeline()
        startStylePipeline()
        startPositionPipeline()
        startAutoTogglePipeline()
    }

    private data class VisibilitySettings(
        val showFps: Boolean = true,
        val showRamPercent: Boolean = false,
        val showRamGb: Boolean = false,
        val showBatTemp: Boolean = false,
        val showCpuTemp: Boolean = false,
        val isVertical: Boolean = false,
    )

    private fun startDataPipeline() {
        val visibilityFlow = combine(
            overlayRepository.isFpsEnabled,
            overlayRepository.isRamPercentageEnabled,
            overlayRepository.isRamGbEnabled,
            overlayRepository.isBatteryTempEnabled,
            overlayRepository.isCpuTempEnabled,
            overlayRepository.isVerticalLayout,
        ) { values ->
            VisibilitySettings(
                showFps = values[0],
                showRamPercent = values[1],
                showRamGb = values[2],
                showBatTemp = values[3],
                showCpuTemp = values[4],
                isVertical = values[5],
            )
        }.onStart {
            emit(VisibilitySettings())
        }

        val stateFlow = combine(
            fpsMonitor.framesPerSecond,
            BatteryUtils.getBatteryFlow(this).onStart {
                BatteryUtils.getBatteryIntent(this@SystemOverlayService)?.let { emit(it) }
            },
            visibilityFlow,
        ) { fps, batteryIntent, vis ->
            Triple(fps, batteryIntent, vis)
        }.stateIn(
            scope = serviceScope,
            started = SharingStarted.Eagerly,
            initialValue = Triple(0, BatteryUtils.getBatteryIntent(this), VisibilitySettings()),
        )

        overlayRepository.overlayUpdateInterval
            .flatMapLatest { interval ->
                flow {
                    while (true) {
                        emit(Unit)
                        delay(interval)
                    }
                }
            }
            .map { stateFlow.value }
            .flowOn(Dispatchers.Default)
            .map { (fps, batteryIntent, vis) ->

                val metrics = mutableListOf<String>()

                if (vis.showFps) {
                    metrics.add(getString(R.string.overlay_format_fps, fps.toLong()))
                }

                if (vis.showRamGb || vis.showRamPercent) {
                    val ram = MemoryUtils.getRamData()
                    val ramText = when {
                        vis.showRamGb && vis.showRamPercent -> getString(
                            R.string.overlay_format_ram_gb_percent,
                            ram.used,
                            ram.total,
                            ram.usedPercentage,
                        )

                        vis.showRamGb -> getString(R.string.overlay_format_ram_gb, ram.used, ram.total)

                        vis.showRamPercent -> getString(R.string.overlay_format_ram_percent, ram.usedPercentage)

                        else -> ""
                    }
                    if (ramText.isNotEmpty()) metrics.add(ramText)
                }

                if (vis.showBatTemp && batteryIntent != null) {
                    val temp = BatteryUtils.getTemperature(batteryIntent)
                    metrics.add(getString(R.string.overlay_format_battery_temp, temp))
                }

                if (vis.showCpuTemp) {
                    val cpuData = CpuUtils.getCpuDynamicData()
                    if (cpuData.isNotEmpty()) {
                        metrics.add(getString(R.string.overlay_format_cpu_temp, cpuData[0]))
                    }
                }

                val separator = if (vis.isVertical) "\n" else " | "
                metrics.joinToString(separator)
            }
            .flowOn(Dispatchers.Default)
            .distinctUntilChanged()
            .onEach { formattedText ->
                metricsView?.text = formattedText
            }
            .launchIn(serviceScope)
    }

    private fun startStylePipeline() {
        combine(
            overlayRepository.overlayTextSize,
            overlayRepository.overlayBgOpacity,
            overlayRepository.overlayPadding,
            overlayRepository.overlayTextColor,
            overlayRepository.overlayCornerRadius,
        ) { size, opacity, padding, color, radius ->
            OverlayStyle(size, opacity, padding, color, radius)
        }
            .distinctUntilChanged()
            .onEach { style ->
                applyStyle(style)
            }
            .launchIn(serviceScope)
    }

    private fun startPositionPipeline() {
        combine(
            overlayRepository.overlayPosition,
            overlayRepository.overlayX,
            overlayRepository.overlayY,
        ) { position, x, y ->
            Triple(position, x, y)
        }
            .distinctUntilChanged()
            .onEach { (pos, x, y) ->
                updateOverlayPosition(pos, x, y)
            }
            .launchIn(serviceScope)
    }

    private fun updateOverlayPosition(position: OverlayPosition, repoX: Int, repoY: Int) {
        val params = overlayView?.layoutParams as? WindowManager.LayoutParams ?: return
        var changed = false

        when (position) {
            OverlayPosition.FREE -> {
                params.gravity = Gravity.TOP or Gravity.START
                params.x = repoX
                params.y = repoY
                params.flags = params.flags and WindowManager.LayoutParams.FLAG_NOT_TOUCHABLE.inv()
                changed = true
            }

            OverlayPosition.TOP_LEFT -> {
                params.gravity = Gravity.TOP or Gravity.START
                params.x = 0
                params.y = 0
                params.flags = params.flags or WindowManager.LayoutParams.FLAG_NOT_TOUCHABLE
                changed = true
            }

            OverlayPosition.TOP_CENTER -> {
                params.gravity = Gravity.TOP or Gravity.CENTER_HORIZONTAL
                params.x = 0
                params.y = 0
                params.flags = params.flags or WindowManager.LayoutParams.FLAG_NOT_TOUCHABLE
                changed = true
            }

            OverlayPosition.TOP_RIGHT -> {
                params.gravity = Gravity.TOP or Gravity.END
                params.x = 0
                params.y = 0
                params.flags = params.flags or WindowManager.LayoutParams.FLAG_NOT_TOUCHABLE
                changed = true
            }
        }
        if (changed) {
            windowManager?.updateViewLayout(overlayView, params)
        }
    }

    private var lastQueryTime: Long = 0L
    private var lastForegroundApp: String? = null

    private fun startAutoTogglePipeline() {
        combine(
            overlayRepository.isAutoToggleEnabled,
            overlayRepository.autoToggleApps,
        ) { enabled, apps ->
            Pair(enabled, apps)
        }
            .flatMapLatest { (enabled, apps) ->
                if (enabled) {
                    overlayRepository.overlayUpdateInterval.flatMapLatest { interval ->
                        flow {
                            while (true) {
                                emit(apps)
                                delay(interval)
                            }
                        }
                    }
                } else {
                    flow {
                        emit(null)
                    }
                }
            }
            .flowOn(Dispatchers.Default)
            .onEach { apps ->
                if (apps == null) {
                    // Auto-toggle disabled, always show
                    serviceScope.launch(Dispatchers.Main) {
                        overlayView?.visibility = View.VISIBLE
                    }
                } else {
                    // Auto-toggle enabled, check foreground app
                    val usageStatsManager = getSystemService(Context.USAGE_STATS_SERVICE) as UsageStatsManager
                    val now = System.currentTimeMillis()
                    if (lastQueryTime == 0L) {
                        lastQueryTime = now - 1000 * 3600 // 1 hour ago for initial
                    }
                    val events = usageStatsManager.queryEvents(lastQueryTime, now)
                    val event = UsageEvents.Event()
                    while (events.hasNextEvent()) {
                        events.getNextEvent(event)
                        if (event.eventType == UsageEvents.Event.ACTIVITY_RESUMED) {
                            lastForegroundApp = event.packageName
                        }
                    }
                    lastQueryTime = now

                    val shouldShow = lastForegroundApp != null && apps.contains(lastForegroundApp)
                    serviceScope.launch(Dispatchers.Main) {
                        overlayView?.visibility = if (shouldShow) View.VISIBLE else View.GONE
                    }
                }
            }
            .launchIn(serviceScope)
    }

    private data class OverlayStyle(val size: Float, val opacity: Float, val padding: Int, val color: Int, val radius: Int)

    private fun applyStyle(style: OverlayStyle) {
        metricsView?.style = style
    }

    override fun onDestroy() {
        super.onDestroy()
        isRunning = false
        serviceScope.cancel()
        if (overlayView != null) {
            windowManager?.removeView(overlayView)
        }
    }

    private fun showOverlay() {
        windowManager = getSystemService(WINDOW_SERVICE) as WindowManager

        val params = WindowManager.LayoutParams(
            WindowManager.LayoutParams.WRAP_CONTENT,
            WindowManager.LayoutParams.WRAP_CONTENT,
            WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY,
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or WindowManager.LayoutParams.FLAG_LAYOUT_IN_SCREEN,
            PixelFormat.TRANSLUCENT,
        ).apply {
            gravity = Gravity.TOP or Gravity.START
            x = 100
            y = 100
        }

        val surfaceView = MetricsSurfaceView(this)

        surfaceView.setOnTouchListener(object : View.OnTouchListener {
            private var initialX = 0
            private var initialY = 0
            private var initialTouchX = 0f
            private var initialTouchY = 0f

            override fun onTouch(v: View, event: MotionEvent): Boolean {
                when (event.action) {
                    MotionEvent.ACTION_DOWN -> {
                        initialX = params.x
                        initialY = params.y
                        initialTouchX = event.rawX
                        initialTouchY = event.rawY
                        return true
                    }

                    MotionEvent.ACTION_MOVE -> {
                        params.x = initialX + (event.rawX - initialTouchX).toInt()
                        params.y = initialY + (event.rawY - initialTouchY).toInt()
                        windowManager?.updateViewLayout(v, params)
                        return true
                    }

                    MotionEvent.ACTION_UP -> {
                        serviceScope.launch {
                            overlayRepository.setOverlayX(params.x)
                            overlayRepository.setOverlayY(params.y)
                        }
                        v.performClick()
                        return true
                    }
                }
                return false
            }
        })

        metricsView = surfaceView
        overlayView = surfaceView
        windowManager?.addView(overlayView, params)
    }

    private fun createNotification(): Notification {
        val channelId = "system_overlay_channel"
        val channel = NotificationChannel(
            channelId,
            getString(R.string.notification_channel_overlay),
            NotificationManager.IMPORTANCE_LOW,
        )
        val notificationManager = getSystemService(NotificationManager::class.java)
        notificationManager.createNotificationChannel(channel)

        return NotificationCompat.Builder(this, channelId)
            .setContentTitle(getString(R.string.notification_title_overlay_active))
            .setContentText(getString(R.string.notification_text_overlay))
            .setSmallIcon(R.drawable.ic_launcher_foreground)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .build()
    }

    companion object {
        private const val NOTIFICATION_ID = 1001
        var isRunning = false
    }

    private inner class MetricsSurfaceView(context: android.content.Context) :
        android.view.SurfaceView(context),
        android.view.SurfaceHolder.Callback {
        private var lastMeasuredWidth = 0
        private var lastMeasuredHeight = 0

        var style: OverlayStyle? = null
            set(value) {
                field = value
                post { requestLayout() }
                drawMetrics()
            }

        var text: String = ""
            set(value) {
                val oldLines = field.split("\n").size
                val newLines = value.split("\n").size
                field = value
                if (oldLines != newLines) {
                    post { requestLayout() }
                } else {
                    val s = style
                    if (s != null) {
                        textPaint.textSize = android.util.TypedValue.applyDimension(
                            android.util.TypedValue.COMPLEX_UNIT_SP,
                            s.size,
                            context.resources.displayMetrics,
                        )
                        var maxWidth = 0f
                        val lines = value.split("\n")
                        for (line in lines) {
                            val w = textPaint.measureText(line)
                            if (w > maxWidth) maxWidth = w
                        }
                        val paddingX = s.padding.toFloat()
                        val neededWidth = (maxWidth + paddingX * 2).toInt()
                        if (neededWidth > lastMeasuredWidth) {
                            post { requestLayout() }
                        } else {
                            drawMetrics()
                        }
                    } else {
                        drawMetrics()
                    }
                }
            }

        private val textPaint = android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG)
        private val bgPaint = android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG)
        private val bgRect = android.graphics.RectF()

        init {
            holder.addCallback(this)
            setZOrderOnTop(true)
            holder.setFormat(android.graphics.PixelFormat.TRANSLUCENT)
        }

        override fun surfaceCreated(holder: android.view.SurfaceHolder) {
            drawMetrics()
        }

        override fun surfaceChanged(holder: android.view.SurfaceHolder, format: Int, width: Int, height: Int) {
            drawMetrics()
        }

        override fun surfaceDestroyed(holder: android.view.SurfaceHolder) {}

        private fun drawMetrics() {
            val s = style ?: return
            val canvas = holder.lockCanvas() ?: return
            try {
                canvas.drawColor(android.graphics.Color.TRANSPARENT, android.graphics.PorterDuff.Mode.CLEAR)

                textPaint.color = s.color
                textPaint.textSize = android.util.TypedValue.applyDimension(
                    android.util.TypedValue.COMPLEX_UNIT_SP,
                    s.size,
                    context.resources.displayMetrics,
                )

                val alphaInt = (s.opacity * 255).toInt()
                bgPaint.color = android.graphics.Color.argb(alphaInt, 0, 0, 0)

                val lines = text.split("\n")
                var maxWidth = 0f
                for (line in lines) {
                    val w = textPaint.measureText(line)
                    if (w > maxWidth) maxWidth = w
                }

                val fm = textPaint.fontMetrics
                val lineHeight = fm.descent - fm.ascent

                val paddingX = s.padding.toFloat()
                val paddingY = (s.padding / 2).toFloat()

                val totalWidth = maxWidth + paddingX * 2
                val totalHeight = (lineHeight * lines.size) + paddingY * 2

                bgRect.set(0f, 0f, totalWidth, totalHeight)
                val radius = s.radius.toFloat()
                canvas.drawRoundRect(bgRect, radius, radius, bgPaint)

                var y = paddingY - fm.ascent
                for (line in lines) {
                    canvas.drawText(line, paddingX, y, textPaint)
                    y += lineHeight
                }
            } finally {
                holder.unlockCanvasAndPost(canvas)
            }
        }

        override fun onMeasure(widthMeasureSpec: Int, heightMeasureSpec: Int) {
            val s = style
            if (s == null) {
                super.onMeasure(widthMeasureSpec, heightMeasureSpec)
                return
            }

            textPaint.textSize = android.util.TypedValue.applyDimension(
                android.util.TypedValue.COMPLEX_UNIT_SP,
                s.size,
                context.resources.displayMetrics,
            )

            val lines = text.split("\n")
            var maxWidth = 0f
            for (line in lines) {
                val w = textPaint.measureText(line)
                if (w > maxWidth) maxWidth = w
            }

            val fm = textPaint.fontMetrics
            val lineHeight = fm.descent - fm.ascent

            val paddingX = s.padding.toFloat()
            val paddingY = (s.padding / 2).toFloat()

            // add a buffer to width to prevent frequent resizing if numbers change
            val totalWidth = (maxWidth + paddingX * 2 + 50f).toInt()
            val totalHeight = ((lineHeight * lines.size) + paddingY * 2).toInt()

            lastMeasuredWidth = totalWidth
            lastMeasuredHeight = totalHeight

            setMeasuredDimension(totalWidth, totalHeight)
        }
    }
}
