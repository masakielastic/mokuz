// build.rs

fn main() {
    // wrapper.c をコンパイル
    cc::Build::new()
        .file("wrapper.c")
        .include("/usr/include/php/20240924")            // `php.h` や `php_embed.h` のパス
        .include("/usr/include/php/20240924/main")       // `php_main.h` のパス
        .include("/usr/include/php/20240924/Zend")       // `zend_string.h` など
        .include("/usr/include/php/20240924/TSRM")       // スレッドセーフ用マクロのためのヘッダ
        .include("/usr/include/php/20240924/sapi/embed")
        .flag_if_supported("-std=c99")
        .compile("php_wrapper");

    // libphp.so をリンク
    println!("cargo:rustc-link-search=native=/usr/lib");
    println!("cargo:rustc-link-lib=dylib=php");
}
