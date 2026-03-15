# j-law-java

日本の法令に基づく各種計算を提供する Java / JVM バインディングです。

`j-law-c-ffi` の C ABI を JNI 経由でラップし、`LocalDate` と `long` ベースの API を提供します。
公開 API は Java 8 互換です。

## インストール

```xml
<dependency>
  <groupId>io.github.kmoyashi</groupId>
  <artifactId>j-law-java</artifactId>
  <version>0.0.1</version>
</dependency>
```

## 使用例

```java
import io.github.kmoyashi.jlaw.BrokerageFeeResult;
import io.github.kmoyashi.jlaw.RealEstate;
import java.time.LocalDate;

BrokerageFeeResult result = RealEstate.calcBrokerageFee(
    5_000_000L,
    LocalDate.of(2024, 8, 1),
    false,
    false
);

System.out.println(result.getTotalWithoutTax()); // 210000
System.out.println(result.getTotalWithTax());    // 231000
System.out.println(result.getTaxAmount());       // 21000
```

## ネイティブライブラリのロード

通常は JAR に同梱された JNI ライブラリが自動的に展開・ロードされます。
開発時に明示したい場合は、以下のいずれかを使えます。

```sh
java -Djlaw.native.lib=/absolute/path/to/libjlaw_jni.dylib ...
```

```sh
export JLAW_JAVA_NATIVE_LIB=/absolute/path/to/libjlaw_jni.so
```

## ビルドとテスト

```sh
cd crates/j-law-java
./gradlew test --no-daemon
```
