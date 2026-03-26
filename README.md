# 6IDE7

<div align="center">

**Интегрированная среда разработки с визуальным программированием**

*Создавайте приложения методом перетаскивания блоков кода*

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Druid](https://img.shields.io/badge/GUI-Druid-blue.svg)](https://github.com/linebender/druid)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

</div>

---

## Описание

**6IDE7** — это инновационная IDE, предназначенная для создания программных приложений методом визуального программирования. Пользователи могут перетаскивать готовые блоки кода на рабочую область и соединять их между собой, формируя логику приложения без необходимости написания кода вручную.

## Ключевые особенности

- 🎨 **Современный GUI на Druid** — отзывчивый и эстетичный интерфейс
- 🖱️ **Drag-and-Drop** — перетаскивание блоков из библиотеки на холст
- 🔗 **Система соединений** — визуальное связывание блоков с проверкой типов
- ✅ **Валидация типов** — проверка совместимости портов при соединении
- ⚡ **Высокая производительность** — реализация на Rust
- 🔄 **Undo/Redo** — история изменений с возможностью отката
- 🔄 **Генерация кода** — автоматическое создание кода на Python, JavaScript, TypeScript, Rust, C++
- 🔌 **Расширяемость** — система плагинов для добавления новых блоков
- 🖥️ **Кроссплатформенность** — Windows, macOS, Linux

## Скриншоты интерфейса

```
┌─────────────────────────────────────────────────────────────────────┐
│  ● Untitled Project  │  ⟷  ✋  ⤫  ✕  │     ▶ Run  ⚙ Settings  💾 Save │
├─────────────┬───────────────────────────────────────────────────────┤
│ ◨ Blocks    │                                                       │
│ ▼ I/O       │     ┌──────────────┐                                  │
│   • Print   │     │   Print      │                                  │
│   • Read    │     │  ○ value     │                                  │
│ ▼ Data      │     └──────────────┘                                  │
│   • Variable│            │                                          │
│   • Constant│     ┌──────▼─────────┐                                 │
│ ▶ Control   │     │     Add        │                                 │
│ ▶ Functions │     │  ○ a    result ○│                                │
│ ▶ Math      │     │  ○ b           │                                 │
│ ▶ Strings   │     └────────────────┘                                 │
├─────────────┴───────────────────────────────────────────────────────┤
│ Output │ Problems │ Terminal                                        │
│ ℹ 6IDE7 Output Console                                               │
│ ℹ Ready to run your program...                                       │
└─────────────────────────────────────────────────────────────────────┘
```

## Архитектура

```
┌─────────────────────────────────────────────────────────┐
│                   Presentation Layer                     │
│         (UI Components, Event Handlers, Theme)          │
├─────────────────────────────────────────────────────────┤
│                   Application Layer                      │
│     (Editor Controller, Project Manager, State)         │
├─────────────────────────────────────────────────────────┤
│                     Domain Layer                         │
│       (Block Models, Connection Graph, Templates)       │
├─────────────────────────────────────────────────────────┤
│                 Infrastructure Layer                     │
│    (File System, Plugin Loader, Code Generator)         │
└─────────────────────────────────────────────────────────┘
```

## Компоненты GUI

| Компонент | Описание |
|-----------|----------|
| **Canvas** | Рабочая область для размещения и соединения блоков с поддержкой zoom/pan |
| **Toolbar** | Панель инструментов с выбором инструментов и действиями проекта |
| **Sidebar** | Боковая панель с библиотекой 30+ блоков и структурой проекта |
| **Output Panel** | Панель вывода с вкладками Output/Problems/Terminal |
| **Settings Dialog** | Диалог настроек с разделами Editor/CodeGen/Appearance/Keybindings |
| **Connection Graph** | Граф соединений с поиском циклов и топологической сортировкой |
| **Type System** | Система типов с проверкой совместимости портов |

## Система блоков

### Типы данных портов

| Тип | Описание | Цвет |
|-----|----------|------|
| Integer | Целое число | Синий |
| Float | Вещественное число | Синий |
| String | Строка | Зелёный |
| Boolean | Логическое значение | Янтарный |
| Array | Массив | Розовый |
| Function | Функция | Фиолетовый |
| Control Flow | Поток выполнения | Янтарный |
| Any | Любой тип | Серый |

### Категории блоков (30+ блоков)

| Категория | Блоки |
|-----------|-------|
| **I/O** | Print, Input, Read File, Write File |
| **Data** | Variable, Constant, Array, Get Item |
| **Control** | If, For Loop, While Loop, For Each, Break, Continue |
| **Functions** | Function, Call, Return |
| **Math** | Add, Subtract, Multiply, Divide, Modulo, Power, Compare, Greater, Less, And, Or, Not |
| **Strings** | Concat, Format, Split, Join, Length |

## Начало работы

### Требования

- Rust 1.75 или выше
- Cargo

### Сборка

```bash
# Клонирование репозитория
git clone https://github.com/vfyov6621-coder/6IDE7.git

# Переход в директорию
cd 6IDE7

# Сборка проекта
cargo build --release

# Запуск
cargo run --release
```

### Структура проекта

```
6IDE7/
├── Cargo.toml              # Workspace configuration
├── crates/
│   ├── ide-app/            # Main application
│   │   ├── src/main.rs     # Entry point
│   │   └── Cargo.toml
│   └── ide-ui/             # UI components
│       ├── src/
│       │   ├── lib.rs      # Module exports
│       │   ├── theme.rs    # Color theme & styling
│       │   ├── canvas.rs   # Block workspace with drag-and-drop
│       │   ├── toolbar.rs  # Top toolbar
│       │   ├── sidebar.rs  # Block library with drag support
│       │   ├── output.rs   # Output console
│       │   ├── settings.rs # Settings dialog
│       │   ├── blocks.rs   # Block definitions (30+ blocks)
│       │   ├── types.rs    # Type system & port specifications
│       │   ├── graph.rs    # Connection graph management
│       │   ├── history.rs  # Undo/Redo system
│       │   └── widgets/    # Reusable widgets
│       └── Cargo.toml
├── docs/                   # Documentation
└── assets/                 # Icons and resources
```

## Горячие клавиши

| Действие | Горячие клавиши |
|----------|-----------------|
| Save | Ctrl+S |
| Undo | Ctrl+Z |
| Redo | Ctrl+Y |
| Delete | Del |
| Select Tool | V |
| Pan Tool | H |
| Connect Tool | C |
| Run Program | F5 |
| Toggle Grid | Ctrl+G |
| Zoom In | Ctrl++ |
| Zoom Out | Ctrl+- |

## Тема оформления

6IDE7 использует современную тёмную тему с цветовой палитрой:

| Элемент | Цвет |
|---------|------|
| Background | `#1e1e2e` |
| Surface | `#25253a` |
| Accent | `#7c3aed` (Purple) |
| I/O Blocks | `#3b82f6` (Blue) |
| Data Blocks | `#22c55e` (Green) |
| Control Blocks | `#f59e0b` (Amber) |
| Function Blocks | `#a855f7` (Purple) |
| Math Blocks | `#ec4899` (Pink) |

## Документация

- 📄 [Архитектурная документация (PDF)](docs/6IDE7_Architecture.pdf)
- 📖 [Концепция и архитектура](docs/ARCHITECTURE.md)

## Статус проекта

🚧 **В разработке** — Проект находится на стадии реализации GUI.

## Roadmap

| Фаза | Период | Цели |
|------|--------|------|
| ✅ Alpha 0.1 | Выполнено | Базовый GUI на Druid |
| ✅ Alpha 0.2 | Выполнено | Drag-and-drop, система соединений, типы данных |
| 🔄 Alpha 0.3 | В процессе | Генерация кода Python/JS |
| ⏳ Beta | Планируется | Все функции, плагины |
| ⏳ RC | Планируется | Импорт кода, отладка |
| ⏳ Release | Планируется | Документация, тестирование, релиз 1.0 |

## Вклад в проект

Мы приветствуем вклад в развитие проекта! Пожалуйста, ознакомьтесь с руководством по внесению изменений.

## Лицензия

MIT License

---

<div align="center">
Создано с ❤️ на Rust с Druid
</div>
