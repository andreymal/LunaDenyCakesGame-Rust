# Принцесса Луна против тортиков

Игра [LunaDenyCakesGame](https://github.com/tereshenkovav/LunaDenyCakesGame), переписанная
на Rust.


## Скачать и поиграть

Готовые сборки для 64-битных Windows, Linux и Android есть
[в разделе Releases на GitHub](https://github.com/andreymal/LunaDenyCakesGame-Rust/releases).

Некоторые ситемные зависимости для Linux-сборок (точный набор зависит от конкретной сборки) —
glibc 2.17+, libasound2, libexpat, libXrandr2, libXcursor1, libxi6 и собственно сам X-сервер
с работающим OpenGL (обычно это всё установлено из коробки в любом приличном Linux-дистрибутиве).

Windows-сборка macroquad, похоже, работает только в Windows 10/11, а SDL и SFML должны работать
и в Windows 7.


## Сборка из исходников

Предполагается выполнение сборки в линуксе с архитектурой x86_64. В других окружениях, вероятно,
тоже может заработать, но это не точно.

Системные зависимости:

* [Rust](https://www.rust-lang.org/tools/install);
* Для бэкенда macroquad — libasound2;
* Для бэкенда SDL — собственно сам SDL2, SDL_image, SDL_mixer, SDL_ttf и их зависимости;
* Для бэкенда SFML — собственно сам SFML и его зависимости;
* GCC для интеграции с сишными библиотеками.

В macroquad и miniquad пришлось фиксить баги, поэтому нужны их локальные пропатченные версии:

    git clone https://github.com/not-fl3/miniquad
    cd miniquad
    git reset --hard 0fa2e35  # Более новые версии не совместимы с Android <= 10
    git apply ../miniquad.patch
    cd ..

    git clone https://github.com/not-fl3/macroquad
    cd macroquad
    git apply ../macroquad.patch
    cd ..

Сборка сама по себе тривиальна, хотя сишные библиотеки могут доставить проблем:

    cargo build --release --bin luna_deny_cakes_game_macroquad --features macroquad
    cargo build --release --bin luna_deny_cakes_game_sdl --features sdl
    cargo build --release --bin luna_deny_cakes_game_sfml --features sfml

Готовые бинарники появятся где-то внутри `target/release` (или, если убрать опцию `--release`,
внутри `target/debug` появятся отладочные сборки).

Можно заменить `cargo build` на `cargo run`, чтобы запустить игру сразу после сборки.

В текущем каталоге должен быть подкаталог `data`, из которого игра будет загружать ассеты.


### Кросс-компиляция

Для уменьшения боли и страданий подготовлены Docker-контейнеры со всем нужным для кросс-компиляции.
(Они чистят каталог `target/release` перед началом сборки, поэтому не храните в нём ничего
ценного.)


#### Windows

Собираем образ на основе Debian, внутри которого будут Rust, MinGW, SDL и SFML:

    docker build -t cakes-cross-windows docker/cross-windows

Запускаем контейнер с этим образом, прицепив к нему исходники через volume, и начнётся сборка:

    docker run --rm -v .:/root/src cakes-cross-windows

(опционально можно добавить что-нибудь вроде `-v /tmp/registry:/usr/local/cargo/registry`
для ускорения повторной сборки)

После чего где-то внутри каталога `target/x86_64-pc-windows-gnu/release` появятся exe и dll,
нужные для запуска игры.


#### Linux

Образ [manylinux2014](https://github.com/pypa/manylinux) предоставляет окружение с достаточно
старой стандартной библиотекой (CentOS 7, glibc 2.17) и достаточно новым компилятором (GCC 10),
что удобно для сборки программы под большинство актуальных линуксов.

На его основе создаём свой образ с доустановленными Rust, SDL, SFML и их зависимостями:

    docker build -t cakes-cross-linux docker/cross-linux

И запускаем сборку аналогично Windows:

    docker run --rm -v .:/root/src cakes-cross-linux

Готовые бинарники и .so-файлы к ним появятся где-то внутри каталога
`target/x86_64-unknown-linux-gnu/release`.


#### Android

Для сборки macroquad-приложения под Android есть готовый контейнер
[notfl3/cargo-apk](https://hub.docker.com/r/notfl3/cargo-apk), содержащий в себе Java 8,
Android NDK и прочие нужные для сборки ништяки, но его пришлось пропатчить для поддержки более
старых версий Android.

    docker build -t cakes-cross-android docker/cross-android

Запуск сборки как обычно:

    docker run --rm -v .:/root/src cakes-cross-android

Готовый apk-файл появится где-то внутри `android_macroquad/target/android-artifacts/release/apk`.

По умолчанию cargo-apk подписывает apk-файл каким-то рандомным отладочным ключом. Чтобы подписать
его своим ключом, нужно собрать его без подписи:

    docker run --rm -v .:/root/src cakes-cross-android build-nosign.sh

А дальше можно использовать утилиты `keytool` и `apksigner` (они тоже есть в образе) как это
обычно делают. Например, если имеется `android.keystore` и в нём ключ с алиасом
`luna_deny_cakes_game`, можно зайти в контейнер и внутри него подписать apk примерно так:

    docker run --rm -v .:/root/src -it cakes-cross-android /bin/bash
    apksigner sign --ks android.keystore --ks-key-alias luna_deny_cakes_game \
        android_macroquad/target/android-artifacts/release/apk/luna_deny_cakes_game_android_macroquad.apk

Подробнее тут: https://macroquad.rs/articles/android/


### Сборка дистрибутива

Скрипты `dist-windows.sh` и `dist-linux.sh` поместят всё ранее собранное в каталог `dist`,
содержимое которого можно будет упаковать в zip-архив и опубликовать.


### Документация

Команда `cargo doc --no-deps -p cake_engine` соберёт документацию к движку и положит в каталог
`target/doc/cake_engine`.
