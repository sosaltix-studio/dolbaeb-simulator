# Dolbaeb simulator

Проект написан на Rust с ипользованием графического движка Macroquad

### Требования
Для сборки проекта вам понадобится установленный компилятор Rust (`rustc` и менеджер пакетов `cargo`). Если у вас их нет, установите с официального сайта [rustup.rs](https://rustup.rs/).

### Установка и запуск

   ```bash
   git clone https://github.com/sosaltix-studio/dolbaeb-simulator

   cd dolbaeb-simulator
   
   cargo run --release
   ```
   
### Запуск в браузере

   ```bash
   git clone https://github.com/sosaltix-studio/dolbaeb-simulator

   cd dolbaeb-simulator

   rustup target add wasm32-unknown-unknown

   cargo build --release --target wasm32-unknown-unknown
   
   python -m http.server

   ```
Игра будет доступна здесь http://0.0.0.0:8000/
