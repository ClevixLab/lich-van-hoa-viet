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
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import vn.clevix.lichviet.BuildConfig
import vn.clevix.lichviet.R
import vn.clevix.lichviet.data.Lich
import vn.clevix.lichviet.data.TextBlock
import vn.clevix.lichviet.update.UpdatePolicy
import vn.clevix.lichviet.update.Updates

/** Câu mô tả phương pháp âm lịch đang dùng — cùng chữ với METHOD_TEXT của pwa/src/ui/about.ts. */
private val METHOD_TEXT = mapOf(
    "jpl" to "Ngày âm lịch tính từ lịch thiên văn JPL DE441 của NASA JPL (thời điểm trăng mới và tiết khí), với ΔT theo Morrison và Stephenson, rồi quy ra ngày theo múi giờ của từng giai đoạn nêu trên. Với những năm rất xa, kết quả kém chắc chắn hơn vì không ai dự báo chính xác được tốc độ quay của Trái Đất (ΔT). Sau năm 2077, vài ngày có trăng mới hoặc tiết khí rơi sát nửa đêm có thể lệch một ngày so với các lịch khác.",
    "precise" to "Ngày âm lịch tính theo mô hình thiên văn chính xác (Meeus): thời điểm trăng mới và vị trí Mặt Trời được tính đầy đủ, rồi quy ra ngày theo múi giờ của từng giai đoạn nêu trên. Thuật toán Hồ Ngọc Đức, phổ biến trong nhiều lịch Việt, vẫn được giữ trong ứng dụng để đối chiếu; hai cách cho cùng kết quả ở gần như mọi ngày.",
    "hnd" to "Ngày âm lịch tính theo thuật toán Hồ Ngọc Đức (phổ biến trong các lịch Việt), quy ra ngày theo múi giờ của từng giai đoạn nêu trên.",
)

private const val TZ_NOTE = "Trước năm 1929, âm lịch tính theo giờ địa phương Bắc Kinh; 1929–1967 theo múi giờ UTC+8; từ 1968 theo UTC+7 (chuẩn miền Bắc). Miền Nam dùng UTC+8 đến năm 1975, nên một số tháng 1968–1975 có thể khác lịch miền Nam thời ấy."

/** "Ứng dụng giúp bạn" — 8 mục, chữ đậm ở đầu (như bản web 0.2.1, PR #73). */
private val HELPS = listOf(
    "**Tờ lịch mỗi ngày** từ năm 1900 đến năm 3000: ngày âm, Can Chi, tiết khí, giờ hoàng đạo và một câu ca dao, tục ngữ.",
    "**Chọn ngày tốt** cho việc lớn: sao tốt, sao xấu, việc nên làm và nên tránh, theo tuổi của bạn.",
    "**Đổi ngày** âm sang dương và ngược lại, kể cả tháng nhuận.",
    "**Người thân** — ghi lại người đã khuất và người đang sống trong gia đình. Ngày giỗ được tự tính theo âm lịch mỗi năm (giỗ đầu, giỗ hết, giỗ thường), cùng sinh nhật dương lịch, sinh nhật âm lịch và những dịp riêng như tốt nghiệp, ngày cưới.",
    "**Nhắc trước** 1, 3, 5, 10, 15 ngày hoặc số ngày bạn tự chọn, để kịp sắm lễ, báo anh em họ hàng. Điện thoại tự báo bằng thông báo, cả khi ứng dụng đang đóng.",
    "**Xem cả nhà có việc gì** trong tuần, tháng, quý hay cả năm; lọc riêng người đã mất hoặc người đang sống.",
    "**Trang tưởng niệm ngày giỗ**: bia ghi tên, năm sinh năm mất theo Can Chi, ảnh chân dung, bát hương ba nén và đôi nến. Gia đình có thể thêm ảnh mâm lễ của chính nhà mình.",
    "**Thông tin báo tử** khi một người thân qua đời: ai báo tin, mất ở đâu, ghi chú, kèm ảnh tin báo tử hoặc bản chụp giấy chứng tử.",
)

/** "Bắt đầu trong một phút" — 6 bước. */
private val STEPS = listOf(
    "Mở mục **Người thân**, chọn **Hồ sơ**, rồi bấm “+ Thêm người đã mất” hoặc “+ Thêm người đang sống”.",
    "Nhập họ tên, cách gọi (Ông, Bà, Bố, Mẹ…) và quan hệ (ví dụ “Ông nội”). Với người đã mất, nhập ngày mất theo dương lịch hoặc âm lịch — ứng dụng tự đổi và tính ngày giỗ hằng năm.",
    "Chọn nhắc trước mấy ngày, bấm “Lưu”. Lần đầu, ứng dụng xin phép gửi thông báo — hãy cho phép để điện thoại tự nhắc.",
    "Mở **Sắp tới** để xem các ngày giỗ, sinh nhật gần nhất; mở **Theo kỳ** để xem theo tuần, tháng, quý, năm.",
    "Đúng ngày giỗ, bấm “Mở trang tưởng niệm” để thắp nén hương tưởng nhớ.",
    "Vào **Sao lưu** để cất một bản sao có mật khẩu. Nếu muốn ứng dụng Lịch của máy cũng hiện các ngày này, có thể xuất tệp lịch (.ics) — không bắt buộc.",
)

/** "Mẹo để không lỡ ngày nào" — bản Android: thông báo thật, không cần .ics hay "Thêm vào Màn hình chính". */
private val TIPS = listOf(
    "Hãy cho phép ứng dụng gửi thông báo (Người thân › Cài đặt nhắc). Điện thoại tự báo đúng ngày, cả khi ứng dụng đang đóng hay máy vừa khởi động lại; thông báo có thể đến muộn tối đa khoảng một giờ so với giờ đã đặt.",
    "Nếu điện thoại có chế độ tiết kiệm pin mạnh và bạn thấy thông báo đến muộn hơn thế, hãy cho phép ứng dụng chạy nền trong phần cài đặt pin của máy.",
    "Thỉnh thoảng tạo tệp sao lưu và cất ở nơi an toàn (ổ đám mây, máy tính). Tệp này mở được trên máy khác, trên bản web, và trên bản iPhone, Windows sắp ra mắt.",
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
            P(UpdatePolicy.runningVersion(Updates.versionName))
            P(UpdatePolicy.EXPLAIN)
            if (fromPlay) P("Bản cài từ Google Play cũng được cửa hàng tự cập nhật nếu bạn bật tự động cập nhật trong Google Play.")
            Button(
                onClick = { Updates.openStore(ctx) },
                colors = ButtonDefaults.buttonColors(containerColor = c.redDeep, contentColor = Color.White),
            ) { Text(UpdatePolicy.BUTTON) }
        }
        Panel {
            H2("Về ứng dụng")
            Text(
                "Lịch và Văn hóa Việt là cuốn lịch vạn niên miễn phí và cũng là sổ tay gia đình: xem ngày âm dương, chọn ngày tốt, và không bao giờ quên ngày giỗ ông bà, cha mẹ hay sinh nhật người thân. Không quảng cáo, không cần tài khoản, dữ liệu chỉ nằm trên máy của bạn.",
                style = MaterialTheme.typography.bodyLarge, color = c.ink, modifier = Modifier.padding(bottom = 8.dp),
            )
            H3("Ứng dụng giúp bạn")
            HELPS.forEach { Item("•", it) }
            H3("Bắt đầu trong một phút")
            STEPS.forEachIndexed { i, t -> Item("${i + 1}.", t) }
            H3("Mẹo để không lỡ ngày nào")
            TIPS.forEach { Item("•", it) }
        }
        Panel {
            H2("Cách tính lịch")
            P("Từ năm 1968, âm lịch tính theo múi giờ UTC+7 (giờ Hà Nội).")
            P(TZ_NOTE)
            P(METHOD_TEXT[Lich.method] ?: "Ngày âm lịch tính theo phương pháp ${Lich.method}.")
            P("Phần xem ngày (sao tốt, sao xấu, việc nên làm và nên tránh, trực, hoàng đạo, hắc đạo) được biên soạn theo quan niệm truyền thống trong sách Ngọc Hạp Thông Thư. Đây là nét văn hóa dân gian, chỉ có giá trị tham khảo; những quyết định quan trọng xin cân nhắc thêm hoàn cảnh thực tế. Điểm số là quy ước của ứng dụng, không có trong sách.")
            TextButton(onClick = openRules) { Text("Nguồn quy tắc xem ngày", color = c.redInk) }
        }
        Panel {
            H2("Quyền riêng tư")
            P("Ứng dụng không thu thập dữ liệu, không có tài khoản, không quảng cáo và không gửi thông tin đi đâu. Mọi tính toán diễn ra ngay trên máy của bạn. Ứng dụng không xin quyền truy cập mạng và hoạt động hoàn toàn khi không có mạng.")
            P("Hồ sơ người thân, ảnh, thông tin báo tử và tệp đính kèm chỉ được lưu trong bộ nhớ riêng của ứng dụng trên điện thoại này, không đưa vào bản sao lưu tự động của máy. Ảnh được tự bỏ thông tin vị trí và máy chụp. Tệp sao lưu được mã hóa bằng mật khẩu do bạn đặt; ứng dụng không giữ mật khẩu đó.")
            P("Dữ liệu vẫn còn khi bạn đóng ứng dụng, tắt máy hay cập nhật ứng dụng. Dữ liệu sẽ mất nếu bạn gỡ cài đặt ứng dụng hoặc xóa dữ liệu của ứng dụng trong phần cài đặt của máy. Vì vậy hãy sao lưu định kỳ.")
            P("Máy chỉ ghi nhớ thêm cỡ chữ, giao diện sáng tối mà bạn chọn và ngày bạn bấm “Để sau” ở thẻ nhắc cập nhật. Năm sinh nhập ở mục Chọn ngày tốt chỉ dùng trong lúc tính, không được lưu lại.")
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
        // Dòng bản quyền hai dòng (giống bản web): "© 2026 Clevix Lab" chữ serif, © nhỏ 0,82× và nhẹ hơn;
        // dưới là "Phiên bản X.Y.Z" nhỏ, nhạt.
        Column(
            modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(
                buildAnnotatedString {
                    withStyle(SpanStyle(fontSize = 0.82.em, fontWeight = FontWeight.Normal, color = c.ink2)) {
                        append(UpdatePolicy.COPYRIGHT_SIGN)
                    }
                    append(UpdatePolicy.COPYRIGHT_HOLDER)
                },
                fontFamily = Serif,
                fontWeight = FontWeight.SemiBold,
                fontSize = 14.sp,
                letterSpacing = 0.01.em,
                color = c.ink,
                textAlign = TextAlign.Center,
            )
            Text(
                UpdatePolicy.versionLine(Updates.versionName),
                style = MaterialTheme.typography.bodySmall,
                fontSize = 11.sp,
                letterSpacing = 0.04.em,
                color = c.ink3,
                textAlign = TextAlign.Center,
                modifier = Modifier.padding(top = 2.dp),
            )
        }
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
private fun H3(t: String) {
    Text(t, style = MaterialTheme.typography.titleMedium, color = LocalLich.current.redInk, modifier = Modifier.padding(top = 6.dp, bottom = 4.dp).semantics { heading() })
}

/** Một dòng danh sách: dấu ([mark]: "•" hay "1.") + chữ, phần trong **…** in đậm. */
@Composable
private fun Item(mark: String, t: String) {
    val c = LocalLich.current
    Row(Modifier.fillMaxWidth().padding(bottom = 8.dp)) {
        Text(mark, style = MaterialTheme.typography.bodyLarge, color = c.ink2, modifier = Modifier.widthIn(min = 26.dp).padding(end = 4.dp))
        Text(
            buildAnnotatedString {
                t.split("**").forEachIndexed { i, part ->
                    if (i % 2 == 1) withStyle(SpanStyle(fontWeight = FontWeight.SemiBold, color = c.ink)) { append(part) } else append(part)
                }
            },
            style = MaterialTheme.typography.bodyLarge, color = c.ink2, modifier = Modifier.weight(1f),
        )
    }
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
