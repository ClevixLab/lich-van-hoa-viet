package vn.clevix.lichviet.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import vn.clevix.lichviet.BuildConfig
import vn.clevix.lichviet.R
import vn.clevix.lichviet.data.Lich
import vn.clevix.lichviet.data.TextBlock
import vn.clevix.lichviet.update.UpdatePolicy
import vn.clevix.lichviet.update.Updates

private val METHOD_LABEL = mapOf(
    "hnd" to "thuật toán Hồ Ngọc Đức (phổ biến trong các lịch Việt)",
    "precise" to "mô hình thiên văn chính xác hơn (Meeus đầy đủ)",
)

/** Giới thiệu — cùng nội dung với pwa/src/ui/about.ts. */
@Composable
fun AboutScreen(openRules: () -> Unit) {
    val c = LocalLich.current
    val ctx = LocalContext.current
    val fromPlay = remember { Updates.installedFromPlay(ctx) }
    Page {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Image(painterResource(R.mipmap.ic_launcher), contentDescription = null, modifier = Modifier.size(72.dp))
            Spacer(Modifier.width(14.dp))
            Column {
                Text("Lịch và Văn hóa Việt", style = MaterialTheme.typography.headlineMedium, color = c.ink, modifier = Modifier.semantics { heading() })
                Text("Clevix Lab", style = MaterialTheme.typography.titleMedium, color = c.redInk)
                Text("Phiên bản ${BuildConfig.VERSION_NAME} (thử nghiệm)", style = MaterialTheme.typography.bodySmall, color = c.ink3)
            }
        }
        Panel {
            H2("Phiên bản và cập nhật")
            P(UpdatePolicy.runningVersion(Updates.versionName, Updates.buildNumber))
            P(UpdatePolicy.EXPLAIN)
            if (fromPlay) P("Bản cài từ Google Play cũng được cửa hàng tự cập nhật nếu bạn bật tự động cập nhật trong Google Play.")
            Button(
                onClick = { Updates.openStore(ctx) },
                colors = ButtonDefaults.buttonColors(containerColor = c.redDeep, contentColor = Color.White),
            ) { Text(UpdatePolicy.BUTTON) }
        }
        Panel {
            H2("Về ứng dụng")
            P("Lịch âm dương Việt Nam theo múi giờ UTC+7, từ năm 2000 đến năm 3000: ngày âm, Can Chi, tiết khí, giờ hoàng đạo, chọn ngày tốt, và một câu ca dao, tục ngữ cho mỗi ngày.")
            P("Ngày âm lịch tính theo ${METHOD_LABEL[Lich.method] ?: Lich.method}.")
            P("Phần xem ngày (sao tốt, sao xấu, việc nên làm và nên tránh, trực, hoàng đạo, hắc đạo) được biên soạn theo quan niệm truyền thống trong sách Ngọc Hạp Thông Thư. Đây là nét văn hóa dân gian, chỉ có giá trị tham khảo; những quyết định quan trọng xin cân nhắc thêm hoàn cảnh thực tế. Điểm số là quy ước của ứng dụng, không có trong sách.")
            TextButton(onClick = openRules) { Text("Nguồn quy tắc xem ngày", color = c.redInk) }
        }
        Panel {
            H2("Quyền riêng tư")
            P("Ứng dụng không thu thập dữ liệu, không có tài khoản, không quảng cáo và không gửi thông tin đi đâu. Mọi tính toán diễn ra ngay trên máy của bạn.")
            P("Ứng dụng không xin quyền truy cập mạng và hoạt động hoàn toàn khi không có mạng. Máy chỉ ghi nhớ cỡ chữ, giao diện sáng tối mà bạn chọn và ngày bạn bấm "Để sau" ở thẻ nhắc cập nhật.")
            P("Năm sinh nhập ở mục Chọn ngày tốt chỉ dùng trong lúc tính, không được lưu lại.")
        }
        Panel {
            H2("Nguồn ca dao, tục ngữ")
            P(Lich.attribution)
            P("Bộ dữ liệu gồm ${Lich.cadaoTotal} câu, phát hành theo giấy phép CC BY-SA 4.0 (Creative Commons Ghi công – Chia sẻ tương tự). Wikiquote và Wikimedia không liên quan và không xác nhận ứng dụng này.")
        }
        Panel {
            H2("Phông chữ")
            P("Phông chữ tiêu đề là bản rút gọn cho chữ Việt của phông Lora (The Lora Project Authors), cấp phép SIL Open Font License 1.1; bản rút gọn mang tên \"LVHV Serif\" theo điều kiện giấy phép.")
        }
        Text(
            UpdatePolicy.copyright(Updates.versionName),
            style = MaterialTheme.typography.bodySmall,
            color = c.ink3,
            modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp),
            textAlign = TextAlign.Center,
        )
    }
}

/** Nguồn quy tắc xem ngày (content/ngochap/SOURCES.md) và nguồn ca dao (content/cadao/SOURCES.md). */
@Composable
fun RulesScreen(back: () -> Unit) {
    val c = LocalLich.current
    Page {
        Text("Nguồn quy tắc", style = MaterialTheme.typography.headlineLarge, color = c.ink, modifier = Modifier.semantics { heading() })
        P("Theo Ngọc Hạp Thông Thư; tham khảo. Địa chỉ trang được ghi để đối chiếu; ứng dụng không mở các trang này.")
        Panel { Blocks(Lich.ruleSources) }
        Panel {
            H2("Nguồn ca dao, tục ngữ (chi tiết)")
            Blocks(Lich.cadaoSources)
        }
        TextButton(onClick = back) { Text("Về trang Giới thiệu", color = c.redInk) }
    }
}

@Composable
private fun Page(content: @Composable () -> Unit) {
    Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.TopCenter) {
        Column(
            Modifier.widthIn(max = 720.dp).fillMaxWidth().verticalScroll(rememberScrollState()).padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) { content() }
    }
}

@Composable
private fun H2(t: String) {
    Text(t, style = MaterialTheme.typography.titleLarge, color = LocalLich.current.ink, modifier = Modifier.padding(bottom = 6.dp).semantics { heading() })
}

@Composable
private fun P(t: String) {
    Text(t, style = MaterialTheme.typography.bodyLarge, color = LocalLich.current.ink2, modifier = Modifier.padding(bottom = 8.dp))
}

@Composable
private fun Blocks(blocks: List<TextBlock>) {
    val c = LocalLich.current
    blocks.forEach { b ->
        when (b.kind) {
            "h" -> Text(b.text, style = MaterialTheme.typography.titleMedium, color = c.ink, modifier = Modifier.padding(top = 10.dp, bottom = 4.dp).semantics { heading() })
            "li" -> Text("• ${b.text}", style = MaterialTheme.typography.bodyMedium, color = c.ink2, modifier = Modifier.padding(start = 6.dp, bottom = 6.dp))
            else -> Text(b.text, style = MaterialTheme.typography.bodyMedium, color = c.ink2, modifier = Modifier.padding(bottom = 8.dp))
        }
    }
    Spacer(Modifier.height(4.dp))
}
