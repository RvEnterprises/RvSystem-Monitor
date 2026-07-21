package com.rve.systemmonitor.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavController
import com.rve.systemmonitor.R
import com.rve.systemmonitor.domain.model.GPU
import com.rve.systemmonitor.ui.components.ExitUntilCollapsedMediumTopAppBar
import com.rve.systemmonitor.ui.components.ScreenWrapper
import com.rve.systemmonitor.ui.components.card.OverviewCard
import com.rve.systemmonitor.ui.components.card.StandardCard
import com.rve.systemmonitor.ui.components.chip.BadgeChip
import com.rve.systemmonitor.ui.components.item.InfoItem
import com.rve.systemmonitor.ui.components.layout.ScreenLazyColumn
import com.rve.systemmonitor.ui.components.row.TwoColumnInfoRow
import com.rve.systemmonitor.ui.viewmodel.GPUViewModel
import java.util.Locale

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun GPUScreen(navController: NavController, onNavigateBack: () -> Unit, viewModel: GPUViewModel = hiltViewModel()) {
    val gpuInfo by viewModel.gpuInfo.collectAsStateWithLifecycle()
    val scrollBehavior = TopAppBarDefaults.exitUntilCollapsedScrollBehavior()

    ScreenWrapper(
        navController = navController,
    ) {
        Scaffold(
            topBar = {
                ExitUntilCollapsedMediumTopAppBar(
                    title = stringResource(R.string.title_graphics_info),
                    onNavigateBack = onNavigateBack,
                    scrollBehavior = scrollBehavior,
                )
            },
            modifier = Modifier.nestedScroll(scrollBehavior.nestedScrollConnection),
            containerColor = MaterialTheme.colorScheme.background,
        ) { innerPadding ->
            Box(modifier = Modifier.padding(innerPadding)) {
                GPUScreenContent(gpuInfo = gpuInfo)
            }
        }
    }
}

@Composable
private fun GPUScreenContent(gpuInfo: GPU) {
    ScreenLazyColumn {
        item {
            OverviewCard(
                iconResId = R.drawable.view_in_ar_filled,
            ) {
                Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                    Column {
                        Text(
                            text = gpuInfo.renderer,
                            style = MaterialTheme.typography.headlineMedium,
                            fontWeight = FontWeight.Bold,
                        )
                        Text(
                            text = stringResource(R.string.label_by, gpuInfo.vendor),
                            style = MaterialTheme.typography.titleMedium,
                        )
                    }

                    BadgeChip(
                        text = String.format(Locale.US, "%.1f °C", gpuInfo.temperature),
                        containerColor = MaterialTheme.colorScheme.primary,
                        textColor = MaterialTheme.colorScheme.onPrimary,
                    )
                }
            }
        }

        item {
            StandardCard {
                Text(
                    text = stringResource(R.string.gpu_opengl_es),
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.padding(bottom = 16.dp),
                )

                TwoColumnInfoRow(modifier = Modifier.padding(bottom = 16.dp)) {
                    InfoItem(
                        label = stringResource(R.string.gpu_label_version),
                        value = gpuInfo.detailedGlesVersion,
                        modifier = Modifier.weight(1f),
                    )
                    InfoItem(
                        label = stringResource(R.string.gpu_label_shader_version),
                        value = gpuInfo.shadingLanguageVersion,
                        modifier = Modifier.weight(1f),
                    )
                }

                TwoColumnInfoRow {
                    InfoItem(
                        label = stringResource(R.string.gpu_label_max_texture_size),
                        value = if (gpuInfo.maxTextureSize > 0) {
                            stringResource(R.string.gpu_max_texture_size_format, gpuInfo.maxTextureSize)
                        } else {
                            stringResource(R.string.value_unknown)
                        },
                        modifier = Modifier.weight(1f),
                    )
                    InfoItem(
                        label = stringResource(R.string.gpu_label_extensions),
                        value = "${gpuInfo.extensionsCount}",
                        modifier = Modifier.weight(1f),
                    )
                }
            }
        }

        item {
            StandardCard {
                Text(
                    text = stringResource(R.string.gpu_vulkan),
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.padding(bottom = 16.dp),
                )

                TwoColumnInfoRow(modifier = Modifier.padding(bottom = 16.dp)) {
                    InfoItem(
                        label = stringResource(R.string.gpu_label_api_version),
                        value = gpuInfo.vulkanVersion,
                        modifier = Modifier.weight(1f),
                    )
                    InfoItem(
                        label = stringResource(R.string.gpu_label_driver_version),
                        value = gpuInfo.vulkanDriverVersion,
                        modifier = Modifier.weight(1f),
                    )
                }

                TwoColumnInfoRow {
                    InfoItem(
                        label = stringResource(R.string.gpu_label_device_type),
                        value = gpuInfo.deviceType,
                        modifier = Modifier.weight(1f),
                    )
                }
            }
        }

        item {
            StandardCard {
                Text(
                    text = stringResource(R.string.gpu_memory_information),
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.padding(bottom = 16.dp),
                )

                TwoColumnInfoRow {
                    InfoItem(
                        label = stringResource(R.string.gpu_label_shared_memory),
                        value = if (gpuInfo.totalMemoryMb > 0) {
                            stringResource(R.string.gpu_shared_memory_format, gpuInfo.totalMemoryMb)
                        } else {
                            stringResource(R.string.value_unknown)
                        },
                        modifier = Modifier.weight(1f),
                    )
                }
            }
        }
    }
}
