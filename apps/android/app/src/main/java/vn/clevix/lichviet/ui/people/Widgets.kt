@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)

package vn.clevix.lichviet.ui.people

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CheckboxDefaults
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.RadioButton
import androidx.compose.material3.RadioButtonDefaults
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import java.time.LocalDate
import java.time.temporal.ChronoUnit
import vn.clevix.lichviet.people.Occ
import vn.clevix.lichviet.people.PFmt
import vn.clevix.lichviet.people.Photos
import vn.clevix.lichviet.ui.LichColors
import vn.clevix.lichviet.ui.LocalLich
import vn.clevix.lichviet.ui.Serif

/** Khung trắng bo góc (như .panel của PWA). */
@Composable
fun PCard(modifier: Modifier = Modifier, title: String? = null, content: @Composable () -> Unit) {
    val c = LocalLich.current
    Surface(
        modifier = modifier.fillMaxWidth(),
        shape = RoundedCornerShape(14.dp),
        color = c.paper,
        border = BorderStroke(1.dp, c.rule),
    ) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            if (title != null) H2(title)
            content()
        }
    }
}

@Composable
fun H1(text: String, modifier: Modifier = Modifier) {
    val c = LocalLich.current
    Text(text, style = MaterialTheme.typography.headlineMedium, color = c.ink, modifier = modifier.semantics { heading() })
}

@Composable
fun H2(text: String, modifier: Modifier = Modifier) {
    val c = LocalLich.current
    Text(text, style = MaterialTheme.typography.titleLarge, color = c.ink, modifier = modifier.semantics { heading() })
}

@Composable
fun H3(text: String) {
    val c = LocalLich.current
    Text(text, style = MaterialTheme.typography.titleMedium, color = c.redInk, modifier = Modifier.semantics { heading() })
}

@Composable
fun Body(text: String, modifier: Modifier = Modifier, color: Color? = null) {
    val c = LocalLich.current
    Text(text, style = MaterialTheme.typography.bodyLarge, color = color ?: c.ink, modifier = modifier)
}

@Composable
fun Fine(text: String, modifier: Modifier = Modifier) {
    val c = LocalLich.current
    Text(text, style = MaterialTheme.typography.bodyMedium, color = c.ink3, modifier = modifier)
}

@Composable
fun ErrorText(text: String) {
    val c = LocalLich.current
    Text(
        text, style = MaterialTheme.typography.bodyMedium, color = c.red,
        modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite },
    )
}

/** Thẻ thông báo: vàng nhạt (thông tin) hoặc đỏ nhạt (cảnh báo). */
@Composable
fun NoticeBox(text: String, warn: Boolean = false, action: (@Composable () -> Unit)? = null, onClose: (() -> Unit)? = null) {
    val c = LocalLich.current
    Surface(
        Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(12.dp),
        color = if (warn) c.redTint else c.goldTint,
        border = BorderStroke(1.dp, if (warn) c.red.copy(alpha = 0.5f) else c.gold.copy(alpha = 0.45f)),
    ) {
        Column(Modifier.padding(14.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(text, style = MaterialTheme.typography.bodyLarge, color = c.ink)
            if (action != null || onClose != null) {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                    action?.invoke()
                    if (onClose != null) TextButton(onClick = onClose) { Text("Đóng", color = c.ink2) }
                }
            }
        }
    }
}

/** Nút chính hình viên thuốc (một nút chính mỗi khung, TK-20). */
@Composable
fun PrimaryButton(text: String, onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true) {
    val c = LocalLich.current
    Button(
        onClick = onClick, enabled = enabled, modifier = modifier.heightIn(min = 48.dp),
        colors = ButtonDefaults.buttonColors(containerColor = c.redDeep, contentColor = Color.White),
    ) { Text(text, style = MaterialTheme.typography.labelLarge, textAlign = TextAlign.Center) }
}

@Composable
fun SecondaryButton(text: String, onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true, danger: Boolean = false) {
    val c = LocalLich.current
    OutlinedButton(
        onClick = onClick, enabled = enabled, modifier = modifier.heightIn(min = 48.dp),
        border = BorderStroke(1.dp, if (danger) c.red else c.rule),
        colors = ButtonDefaults.outlinedButtonColors(contentColor = if (danger) c.red else c.ink),
    ) { Text(text, style = MaterialTheme.typography.labelLarge, textAlign = TextAlign.Center) }
}

@Composable
fun LinkButton(text: String, onClick: () -> Unit) {
    val c = LocalLich.current
    TextButton(onClick = onClick, modifier = Modifier.heightIn(min = 48.dp)) {
        Text(text, style = MaterialTheme.typography.labelLarge, color = c.redInk)
    }
}

/** Nhóm chọn một trong nhiều (Tuần/Tháng/Quý/Năm, Dương/Âm…). */
@Composable
fun Choice(options: List<String>, selected: Int, onSelect: (Int) -> Unit, modifier: Modifier = Modifier) {
    val c = LocalLich.current
    SingleChoiceSegmentedButtonRow(modifier.fillMaxWidth().height(androidx.compose.foundation.layout.IntrinsicSize.Min)) {
        options.forEachIndexed { i, t ->
            SegmentedButton(
                selected = selected == i,
                onClick = { onSelect(i) },
                shape = SegmentedButtonDefaults.itemShape(i, options.size),
                icon = {},
                colors = SegmentedButtonDefaults.colors(
                    activeContainerColor = c.redDeep, activeContentColor = Color.White,
                    inactiveContainerColor = c.paper, inactiveContentColor = c.ink,
                    activeBorderColor = c.redDeep, inactiveBorderColor = c.rule,
                ),
                modifier = Modifier.heightIn(min = 48.dp).fillMaxHeight(),
            ) { Text(t, maxLines = 2, textAlign = TextAlign.Center, style = MaterialTheme.typography.labelLarge) }
        }
    }
}

/** Nút bật/tắt dạng chip (bộ lọc, mốc nhắc). */
@Composable
fun Chip(text: String, on: Boolean, onClick: () -> Unit, enabled: Boolean = true) {
    val c = LocalLich.current
    FilterChip(
        selected = on, onClick = onClick, enabled = enabled,
        label = { Text(text, style = MaterialTheme.typography.labelLarge) },
        modifier = Modifier.heightIn(min = 48.dp).semantics { stateDescription = if (on) "Đang chọn" else "Không chọn" },
        colors = FilterChipDefaults.filterChipColors(
            selectedContainerColor = c.redDeep, selectedLabelColor = Color.White,
            containerColor = c.paper, labelColor = c.ink,
        ),
    )
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun Flow(content: @Composable () -> Unit) {
    FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) { content() }
}

/** Hộp kiểm có nhãn; vùng chạm là cả hàng (TK-04). */
@Composable
fun CheckRow(text: String, checked: Boolean, onChange: (Boolean) -> Unit, enabled: Boolean = true) {
    val c = LocalLich.current
    Row(
        Modifier.fillMaxWidth().heightIn(min = 48.dp)
            .clickable(enabled = enabled, role = Role.Checkbox) { onChange(!checked) },
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Checkbox(
            checked = checked, onCheckedChange = null, enabled = enabled,
            colors = CheckboxDefaults.colors(checkedColor = c.redDeep, checkmarkColor = Color.White),
        )
        Spacer(Modifier.width(10.dp))
        Text(text, style = MaterialTheme.typography.bodyLarge, color = if (enabled) c.ink else c.ink3)
    }
}

@Composable
fun SwitchRow(text: String, checked: Boolean, onChange: (Boolean) -> Unit, sub: String? = null) {
    val c = LocalLich.current
    Row(
        Modifier.fillMaxWidth().heightIn(min = 56.dp).clickable(role = Role.Switch) { onChange(!checked) },
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f)) {
            Text(text, style = MaterialTheme.typography.bodyLarge, color = c.ink)
            if (sub != null) Fine(sub)
        }
        Spacer(Modifier.width(12.dp))
        Switch(
            checked = checked, onCheckedChange = null,
            colors = SwitchDefaults.colors(checkedTrackColor = c.redDeep, checkedThumbColor = Color.White),
        )
    }
}

@Composable
fun RadioRow(text: String, selected: Boolean, onClick: () -> Unit) {
    val c = LocalLich.current
    Row(
        Modifier.fillMaxWidth().heightIn(min = 48.dp).clickable(role = Role.RadioButton, onClick = onClick),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        RadioButton(selected = selected, onClick = null, colors = RadioButtonDefaults.colors(selectedColor = c.redDeep))
        Spacer(Modifier.width(10.dp))
        Text(text, style = MaterialTheme.typography.bodyLarge, color = c.ink)
    }
}

/** Ô chữ có nhãn luôn thấy, lỗi ngay dưới ô. */
@Composable
fun TextIn(
    label: String,
    value: String,
    onChange: (String) -> Unit,
    modifier: Modifier = Modifier,
    error: String? = null,
    multiline: Boolean = false,
    help: String? = null,
    password: Boolean = false,
    number: Boolean = false,
) {
    Column(modifier.fillMaxWidth()) {
        OutlinedTextField(
            value = value,
            onValueChange = onChange,
            label = { Text(label) },
            singleLine = !multiline,
            minLines = if (multiline) 3 else 1,
            isError = error != null,
            visualTransformation = if (password) androidx.compose.ui.text.input.PasswordVisualTransformation()
            else androidx.compose.ui.text.input.VisualTransformation.None,
            keyboardOptions = when {
                password -> KeyboardOptions(keyboardType = KeyboardType.Password, autoCorrectEnabled = false)
                number -> KeyboardOptions(keyboardType = KeyboardType.Number)
                else -> KeyboardOptions.Default
            },
            modifier = Modifier.fillMaxWidth(),
            colors = fieldColors(),
        )
        if (error != null) ErrorText(error)
        else if (help != null) Fine(help, Modifier.padding(top = 2.dp))
    }
}

/** Ô số nhỏ (ngày / tháng / năm / giờ). */
@Composable
fun NumIn(label: String, value: String, onChange: (String) -> Unit, modifier: Modifier = Modifier, isError: Boolean = false, maxLen: Int = 4) {
    val c = LocalLich.current
    // Nhãn nằm trên ô (không trong viền) để ô hẹp vẫn đọc được nhãn ở cỡ chữ lớn.
    Column(modifier) {
        Text(
            label, style = MaterialTheme.typography.labelLarge, color = if (isError) c.red else c.ink2,
            modifier = Modifier.padding(bottom = 4.dp).clearAndSetSemantics { },
        )
        OutlinedTextField(
            value = value,
            onValueChange = { s -> onChange(s.filter { it.isDigit() }.take(maxLen)) },
            singleLine = true,
            isError = isError,
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
            modifier = Modifier.fillMaxWidth().semantics { contentDescription = label },
            colors = fieldColors(),
        )
    }
}

/** Hộp xác nhận (AlertDialog). Nút chính bên phải; nút an toàn "Hủy"/"Giữ lại". */
@Composable
fun ConfirmDialog(
    title: String,
    confirm: String,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit,
    dismiss: String = "Hủy",
    danger: Boolean = false,
    confirmEnabled: Boolean = true,
    dismissEnabled: Boolean = true,
    body: @Composable () -> Unit,
) {
    val c = LocalLich.current
    AlertDialog(
        onDismissRequest = { if (dismissEnabled) onDismiss() },
        title = { Text(title, style = MaterialTheme.typography.titleLarge, color = c.ink) },
        text = { Column(verticalArrangement = Arrangement.spacedBy(8.dp)) { body() } },
        confirmButton = {
            Button(
                onClick = onConfirm, enabled = confirmEnabled,
                colors = ButtonDefaults.buttonColors(containerColor = if (danger) c.red else c.redDeep, contentColor = Color.White),
            ) { Text(confirm) }
        },
        dismissButton = { TextButton(onClick = onDismiss, enabled = dismissEnabled) { Text(dismiss, color = c.ink) } },
        containerColor = c.paper,
    )
}

/** Vòng tròn chữ cái đầu của tên (danh sách hồ sơ không giải mã ảnh). */
@Composable
fun Initial(name: String, size: Int = 44) {
    val c = LocalLich.current
    val ch = name.trim().substringAfterLast(' ').firstOrNull()?.uppercase() ?: "?"
    Box(
        Modifier.size(size.dp).clip(CircleShape).background(c.redTint),
        contentAlignment = Alignment.Center,
    ) { Text(ch, style = MaterialTheme.typography.titleMedium, color = c.redInk, fontFamily = Serif) }
}

/** Ảnh đã lưu (giải mã một lần). */
@Composable
fun rememberPhoto(bytes: ByteArray?): ImageBitmap? = remember(bytes) {
    bytes?.let { Photos.bitmap(it)?.asImageBitmap() }
}

/** Ảnh trong khung bầu dục. */
@Composable
fun OvalPhoto(img: ImageBitmap?, name: String, w: Int = 120, h: Int = 150, frame: Color? = null) {
    val c = LocalLich.current
    val shape = androidx.compose.foundation.shape.GenericShape { size, _ ->
        addOval(androidx.compose.ui.geometry.Rect(0f, 0f, size.width, size.height))
    }
    Box(
        Modifier.size(w.dp, h.dp).clip(shape).background(c.paper2)
            .border(3.dp, frame ?: c.gold, shape),
        contentAlignment = Alignment.Center,
    ) {
        if (img != null) {
            Image(img, contentDescription = "Ảnh chân dung", contentScale = ContentScale.Crop, modifier = Modifier.size(w.dp, h.dp))
        } else {
            val ch = name.trim().substringAfterLast(' ').firstOrNull()?.uppercase() ?: ""
            Text(ch, style = MaterialTheme.typography.displayLarge, color = c.ink3)
        }
    }
}

/** Khoảng cách tới hôm nay (08 §5.1). */
fun daysText(date: LocalDate, today: LocalDate): String {
    val d = ChronoUnit.DAYS.between(today, date)
    return when {
        d == 0L -> "Hôm nay"
        d == 1L -> "Ngày mai"
        d > 1 -> "Còn $d ngày"
        d == -1L -> "Hôm qua"
        else -> "Đã qua ${-d} ngày"
    }
}

/** Dòng 1 của mục: "Giỗ đầu · Ông Nguyễn Văn A" (tách từ headline của lõi). */
fun occTitle(o: Occ): String {
    val h = o.headline
    return if (o.name.isNotEmpty() && h.endsWith(o.name) && h.length > o.name.length) {
        h.removeSuffix(o.name).trimEnd(' ', '·', '—', ':') + " · " + o.name
    } else if (o.name.isNotEmpty() && !h.contains(o.name)) {
        "$h · ${o.name}" // sự kiện: tên sự kiện · tên người
    } else h
}

/** Dòng 2: "26/11/2026 · 7/10 âm lịch". */
fun occDates(o: Occ): String = PFmt.dmy(o.date) + (o.lunar?.let { " · ${it.dm} âm lịch" } ?: "")

/** Biểu tượng loại lần xuất hiện, vẽ nét (aria-hidden: loại luôn có chữ đi kèm). */
@Composable
fun KindIcon(cls: String, color: Color, modifier: Modifier = Modifier.size(24.dp)) {
    Canvas(modifier) {
        val s = size.minDimension / 24f
        val st = Stroke(width = 1.8f * s, cap = StrokeCap.Round)
        when (cls) {
            "gio" -> {
                for (x in listOf(8f, 12f, 16f)) drawLine(color, Offset(x * s, 5f * s), Offset(x * s, 15f * s), 1.8f * s, StrokeCap.Round)
                drawRoundRect(color, Offset(5f * s, 15f * s), Size(14f * s, 5f * s), androidx.compose.ui.geometry.CornerRadius(2f * s), style = st)
            }
            "tien_thuong" -> {
                drawOval(color, Offset(5f * s, 8f * s), Size(14f * s, 6f * s), style = st)
                drawLine(color, Offset(3f * s, 17f * s), Offset(21f * s, 17f * s), 1.8f * s, StrokeCap.Round)
            }
            "sinh_nhat", "sinh_nhat_am" -> {
                drawRoundRect(color, Offset(5f * s, 11f * s), Size(14f * s, 9f * s), androidx.compose.ui.geometry.CornerRadius(2f * s), style = st)
                for (x in listOf(9f, 12f, 15f)) drawLine(color, Offset(x * s, 6f * s), Offset(x * s, 10f * s), 1.8f * s, StrokeCap.Round)
            }
            else -> {
                val p = androidx.compose.ui.graphics.Path().apply {
                    moveTo(12f * s, 3.5f * s); lineTo(14.5f * s, 8.7f * s); lineTo(20.1f * s, 9.4f * s); lineTo(16f * s, 13.3f * s)
                    lineTo(17f * s, 18.9f * s); lineTo(12f * s, 16.2f * s); lineTo(7f * s, 18.9f * s); lineTo(8f * s, 13.3f * s)
                    lineTo(3.9f * s, 9.4f * s); lineTo(9.5f * s, 8.7f * s); close()
                }
                drawPath(p, color, style = st)
            }
        }
    }
}

/**
 * Mục sự kiện (08 §2): ô ngày dương | nhãn + tên, ngày dương · ngày âm | biểu tượng + khoảng cách.
 * [onClick] = null → chỉ để xem (khung "Lần tới", TK-08).
 */
@Composable
fun OccRow(o: Occ, today: LocalDate, onClick: (() -> Unit)?) {
    val c = LocalLich.current
    val isToday = o.date == today
    val past = o.date.isBefore(today)
    val dist = o.daysText ?: daysText(o.date, today)
    val shape = RoundedCornerShape(12.dp)
    Row(
        Modifier.fillMaxWidth().heightIn(min = 64.dp).clip(shape)
            .background(c.paper)
            .border(if (isToday) 2.dp else 1.dp, if (isToday) c.red else c.rule, shape)
            .let { if (onClick != null) it.clickable(role = Role.Button, onClick = onClick) else it }
            .semantics(mergeDescendants = true) { contentDescription = "${occTitle(o)}. ${occDates(o)}. $dist" }
            .padding(horizontal = 10.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.widthIn(min = 52.dp), horizontalAlignment = Alignment.CenterHorizontally) {
            Text(PFmt.weekdayShort(o.date), style = MaterialTheme.typography.labelMedium, color = if (o.date.dayOfWeek.value == 7) c.red else c.ink2)
            Text("${o.date.dayOfMonth}", style = MaterialTheme.typography.headlineSmall, fontFamily = Serif, color = c.ink)
            Text(PFmt.monthShort(o.date), style = MaterialTheme.typography.labelMedium, color = c.ink2)
        }
        Spacer(Modifier.width(10.dp))
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(
                occTitle(o), style = MaterialTheme.typography.titleMedium, color = if (past) c.ink2 else c.ink,
                maxLines = 3, overflow = TextOverflow.Ellipsis,
            )
            Text(occDates(o), style = MaterialTheme.typography.bodyMedium, color = c.ink2)
        }
        Spacer(Modifier.width(8.dp))
        Column(horizontalAlignment = Alignment.End, modifier = Modifier.widthIn(max = 96.dp)) {
            KindIcon(o.cls, if (o.isGio) c.gold else c.redInk)
            Text(
                dist, style = MaterialTheme.typography.labelMedium,
                color = when { isToday -> c.red; past -> c.ink3; else -> c.ink2 },
                fontWeight = if (isToday) FontWeight.Bold else FontWeight.Normal, textAlign = TextAlign.End,
            )
        }
    }
}

/** Dòng nhãn — giá trị trong bảng thông tin. */
@Composable
fun InfoLine(label: String, value: String) {
    val c = LocalLich.current
    Column(Modifier.fillMaxWidth().padding(vertical = 2.dp)) {
        Text(label, style = MaterialTheme.typography.labelMedium, color = c.ink3)
        Text(value, style = MaterialTheme.typography.bodyLarge, color = c.ink)
    }
}

@Composable
fun Gap(h: Int = 8) = Spacer(Modifier.height(h.dp))

/** Viền ô nhập đủ tương phản (≥ 3:1 với nền, 08 §6). */
@Composable
fun fieldColors(): androidx.compose.material3.TextFieldColors {
    val c = LocalLich.current
    return androidx.compose.material3.OutlinedTextFieldDefaults.colors(
        unfocusedBorderColor = c.ink3,
        focusedBorderColor = c.redInk,
        focusedLabelColor = c.redInk,
        unfocusedLabelColor = c.ink2,
        cursorColor = c.redInk,
        focusedTextColor = c.ink,
        unfocusedTextColor = c.ink,
        errorBorderColor = c.red,
        errorLabelColor = c.red,
    )
}
