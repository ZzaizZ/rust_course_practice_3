# API

Общие типы и protobuf схемы для HTTP REST API и gRPC.

## Структура

```
api/
├── proto/
│   └── blog.proto     # Protobuf схема для gRPC
├── src/
│   ├── lib.rs         # Модули экспорта
│   ├── rest.rs        # REST API типы (serde)
│   └── generated/     # Сгенерированный код из protobuf
└── build.rs           # Генерация кода из .proto файлов
```

## Features

- `rest` - REST API типы с поддержкой serde (включен по умолчанию)
- `grpc` - gRPC типы из protobuf (включен по умолчанию)
- `default` - Включает и `rest`, и `grpc`

## Protobuf схема

Схема определена в `proto/blog.proto`.

## Сборка

При сборке проекта `build.rs` автоматически генерирует Rust код из protobuf схемы:

```bash
cargo build -p api
```

Сгенерированные файлы создаются в `target/`.

## Разработка

При изменении protobuf схемы необходимо пересобрать проект:

```bash
cargo clean -p api
cargo build -p api
```

Изменения в `rest.rs` применяются сразу при следующей компиляции.
