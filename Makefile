.PHONY: help setup build install clean

help:
 @echo "Команды для локальной разработки:"
 @echo "  make setup   - Главная команда. Устанавливает зависимости и компилирует ядро."
 @echo "  make build   - (Пере)собирает Rust-ядро под вашу архитектуру."
 @echo "  make install - Устанавливает/обновляет Python-зависимости."
 @echo "  make clean   - Удаляет артефакты сборки."

# Одна команда, чтобы всё заработало.
# Сначала ставит зависимости, потом компилирует Rust.
setup: install build
 @echo "✅ Готово! Проект настроен и скомпилирован. Можно запускать."

build:
 @echo "⚙️  Компилирую Rust-ядро..."
 docker run --rm -v $(pwd):/io ghcr.io/pyo3/maturin build --release -i python3.11 -o dist
 @echo "📦 Сборка завершена. Wheel-файл в папке dist/."

install:
 @echo "🐍 Устанавливаю Python-зависимости..."
 poetry install

clean:
 @echo "🧹 Очищаю артефакты сборки..."
 rm -rf target/ dist/ *.egg-info build/
