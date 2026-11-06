package vn.clevix.lichviet.data

/**
 * Bộ đọc JSON tối giản cho dữ liệu do lõi Rust sinh ra (lvhv-ffi). Không phụ thuộc org.json để
 * phần dữ liệu biên dịch và kiểm thử được trên JVM thường (ngoài Android).
 *
 * Kết quả: Map<String, Any?> (giữ thứ tự khóa), List<Any?>, String, Long, Double, Boolean, null.
 */
internal class JsonReader private constructor(private val s: String) {
    private var i = 0

    companion object {
        fun parse(text: String): Any? {
            val r = JsonReader(text)
            r.ws()
            val v = r.value()
            r.ws()
            if (r.i != r.s.length) throw IllegalArgumentException("JSON thừa ký tự ở vị trí ${r.i}")
            return v
        }
    }

    private fun ws() {
        while (i < s.length && (s[i] == ' ' || s[i] == '\n' || s[i] == '\r' || s[i] == '\t')) i++
    }

    private fun value(): Any? {
        if (i >= s.length) throw IllegalArgumentException("JSON kết thúc sớm")
        return when (val c = s[i]) {
            '{' -> obj()
            '[' -> arr()
            '"' -> str()
            't' -> lit("true", true)
            'f' -> lit("false", false)
            'n' -> lit("null", null)
            else -> if (c == '-' || c.isDigit()) num() else throw IllegalArgumentException("JSON lỗi ở vị trí $i")
        }
    }

    private fun lit(word: String, v: Any?): Any? {
        if (!s.startsWith(word, i)) throw IllegalArgumentException("JSON lỗi ở vị trí $i")
        i += word.length
        return v
    }

    private fun obj(): Map<String, Any?> {
        val m = LinkedHashMap<String, Any?>()
        i++ // {
        ws()
        if (s[i] == '}') { i++; return m }
        while (true) {
            ws()
            val k = str()
            ws()
            if (s[i] != ':') throw IllegalArgumentException("Thiếu ':' ở vị trí $i")
            i++
            ws()
            m[k] = value()
            ws()
            when (s[i]) {
                ',' -> i++
                '}' -> { i++; return m }
                else -> throw IllegalArgumentException("Thiếu ',' hoặc '}' ở vị trí $i")
            }
        }
    }

    private fun arr(): List<Any?> {
        val a = ArrayList<Any?>()
        i++ // [
        ws()
        if (s[i] == ']') { i++; return a }
        while (true) {
            ws()
            a.add(value())
            ws()
            when (s[i]) {
                ',' -> i++
                ']' -> { i++; return a }
                else -> throw IllegalArgumentException("Thiếu ',' hoặc ']' ở vị trí $i")
            }
        }
    }

    private fun str(): String {
        if (s[i] != '"') throw IllegalArgumentException("Cần chuỗi ở vị trí $i")
        i++
        val b = StringBuilder()
        while (true) {
            val c = s[i++]
            when (c) {
                '"' -> return b.toString()
                '\\' -> {
                    when (val e = s[i++]) {
                        '"' -> b.append('"')
                        '\\' -> b.append('\\')
                        '/' -> b.append('/')
                        'b' -> b.append('\b')
                        'f' -> b.append('\u000C')
                        'n' -> b.append('\n')
                        'r' -> b.append('\r')
                        't' -> b.append('\t')
                        'u' -> {
                            b.append(s.substring(i, i + 4).toInt(16).toChar())
                            i += 4
                        }
                        else -> throw IllegalArgumentException("Thoát chuỗi lạ '\\$e'")
                    }
                }
                else -> b.append(c)
            }
        }
    }

    private fun num(): Any {
        val start = i
        if (s[i] == '-') i++
        while (i < s.length && (s[i].isDigit() || s[i] == '.' || s[i] == 'e' || s[i] == 'E' || s[i] == '+' || s[i] == '-')) i++
        val t = s.substring(start, i)
        return t.toLongOrNull() ?: t.toDouble()
    }
}

@Suppress("UNCHECKED_CAST")
internal fun Any?.obj(): Map<String, Any?> = this as Map<String, Any?>

@Suppress("UNCHECKED_CAST")
internal fun Any?.arr(): List<Any?> = this as List<Any?>

internal fun Map<String, Any?>.int(k: String): Int = (this[k] as Number).toInt()
internal fun Map<String, Any?>.str(k: String): String = this[k] as String
internal fun Map<String, Any?>.bool(k: String): Boolean = this[k] as Boolean
internal fun Map<String, Any?>.strList(k: String): List<String> = (this[k] as List<*>).map { it as String }
