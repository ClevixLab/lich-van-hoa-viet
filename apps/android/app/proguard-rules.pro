# R8 cho Lịch và Văn hóa Việt.
# JNA dùng phản chiếu (reflection) để ánh xạ hàm/cấu trúc gốc; mã UniFFI sinh ra dựa trên JNA.
-keep class com.sun.jna.** { *; }
-keepclassmembers class * extends com.sun.jna.** { public *; }
-dontwarn java.awt.**
-dontwarn com.sun.jna.**

# Mã UniFFI sinh ra: giao diện Library, Structure (RustBuffer, ...), callback.
-keep class vn.clevix.lichviet.core.** { *; }
-keepclassmembers class vn.clevix.lichviet.core.** { *; }

# Không cần thông tin dòng mã trong bản phát hành công khai, nhưng giữ để đọc báo lỗi Play Console.
-keepattributes SourceFile,LineNumberTable,*Annotation*,Signature,InnerClasses,EnclosingMethod
-renamesourcefileattribute SourceFile
