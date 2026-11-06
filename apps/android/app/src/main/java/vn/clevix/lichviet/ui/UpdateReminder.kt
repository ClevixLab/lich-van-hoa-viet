package vn.clevix.lichviet.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import vn.clevix.lichviet.update.UpdatePolicy

/** Thẻ nhắc cập nhật nhẹ nhàng, không chặn thao tác, ở đầu màn hình Tờ lịch. */
@Composable
fun UpdateReminder(onCheck: () -> Unit, onLater: () -> Unit) {
    val c = LocalLich.current
    Box(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp), contentAlignment = Alignment.TopCenter) {
        Column(
            Modifier
                .widthIn(max = 560.dp)
                .fillMaxWidth()
                .clip(RoundedCornerShape(14.dp))
                .background(c.goldTint)
                .padding(horizontal = 14.dp, vertical = 10.dp)
                .semantics { liveRegion = LiveRegionMode.Polite },
        ) {
            Text(UpdatePolicy.REMINDER, style = MaterialTheme.typography.bodyMedium, color = c.ink)
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End, verticalAlignment = Alignment.CenterVertically) {
                TextButton(onClick = onLater) { Text("Để sau", color = c.ink2) }
                Button(
                    onClick = onCheck,
                    colors = ButtonDefaults.buttonColors(containerColor = c.redDeep, contentColor = Color.White),
                ) { Text("Kiểm tra cập nhật") }
            }
        }
    }
}
