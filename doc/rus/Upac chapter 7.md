<!--
SPDX-FileCopyrightText: 2026 JustPav

SPDX-License-Identifier: CC-BY-SA-4.0
-->

## **§7.** Модули программы.

***ВНИМАНИЕ:*** Ориентир для проектирования, уточняется по ходу разработки!

**`lib/`** — ядро программы и FFI (реальная раскладка модулей, поддерживается в актуальном виде по ходу разработки):

- `export` — C-ABI: точки входа всех команд, версия ABI, отмена, освобождение ответов;
- `orchestrator` — общий движок (См. **§5.9**–**§5.11**): `Stage`/`ConcurrentStage`, `Cursor`, `RollbackGuard`, и два оркестратора за одним трейтом `Orchestrator` — `SequentialOrchestrator` (линейный, держит системный лок файл) и `ParallelOrchestrator` (параллельные стадии, используется для хуков в **§5.8**);
- `mutated` / `unmutated` — тела команд, по подмодулю на каждую. Собираются из своего pipeline каждой стадии через `orchestrator`;
- `scripts` — пункт **§5.8**: TOML-формат хук-файла (`HookFile`), примитивы (`Primitive`/`TouchFile`/`MoveFile`/`CreateSymlink`, каждый `impl Step { execute, rollback }`), матчинг нативных триггеров (`Operation`/`Timing`). `HookStage::run()` полностью подключён для нативных триггеров: get-or-build общего `tokio`-рантайма через `Context`, далее проверка подписи и парсинг хук-файлов (`load_hooks`, через `upac-pki`), фильтрация по `NativeTrigger`, параллельный запуск совпавших хуков через `ParallelOrchestrator` (`HookFile` сам `impl ConcurrentStage`, исполняет свои `steps` и учитывает `critical`). Вписан в pipeline всех mutated команд (Pre/Post обработка хуков каждой);
- `plugin` — загрузка декодеров. В данный момент реализован только подмодуль `decoder` (`dlopen`, проверка версии ABI, `decode`/`match_triggers`) — родительский каталог `plugin` зарезервирован под другие виды плагинов на будущее, пока таковых нет. Там же `manifest` (`DecoderManifest`, `load_decoder_manifests()` — читает декларативные файлы для описания декодерв в каталоге `/etc/upac.d/decoders/*.toml`, без сканирования и/или проверки `.so`) и `triggers` (`build_trigger_table()` — строит таблицу native-триггер→хук под конкретный декодер из загруженных `HookFile`, разрешая конфликты `priority` жёсткой ошибкой операции). Пока никуда не подключено — нужна реальная точка вызова, завязанная на ещё не написанные тела стадий каждой команды;
- `composefs` — доступ к composefs-репозиторию. `Repository`: `open(path) -> Repository<ObjectID>` (открытие по пути через `Repository::open_path`), `open_tree(repository, name) -> FileSystem<ObjectID>` (читает образ через `Repository::open_image` + `erofs::reader::erofs_to_filesystem`) — обе доступны только внутри библиотеки, наружу отдаётся только `deploy::Deploy`. `error::RepoError` — маппинг `RepositoryOpenError`/`ImageError`/`anyhow::Error` (последнее нужно, потому что `ensure_object`/`ensure_object_from_file`/`commit_image` и т.п. в самом composefs возвращают `anyhow::Result` — деталей ошибки оттуда не достать, только факт неудачи). `file::FileHandle` — держатель/указатель на путь в дереве, три `impl`-блока по логике "трогает CAS или нет": конструкторы (`new` — слепой, для вставки нового; `from_tree` — с проверкой, что путь уже существует), дерево без CAS (`insert_in_tree`/`update_in_tree`/`rename_in_tree`/`remove_in_tree`/`symlink_in_tree`/`hardlink_in_tree`/`stat_in_tree`/`symlink_target_in_tree`/`list_in_tree`), файл через CAS (`insert_file` берёт уже открытый `&File` — не байты, чтобы путь резолвился ровно один раз и не было TOCTOU-гонки на подмену файла между чтением и вставкой в CAS; `replace_file` — алиас на `insert_file`, т.к. `Directory::insert` в composefs уже сам upsert-ит; `read_file` — резолвит inline/external и тянет байты из `Repository::read_object` при необходимости);
- `deploy` — постановка деплоя (См. **§5.3**): находит блочное устройство под `/` через `MountInfo`, реальный тип ФС — через `rsblkid::probe::Probe` (не `None`, иначе `mount(2)` падает `EINVAL` — тип ФС ядру нужен явно для любого монтирования, кроме bind/remount), `unshare(CLONE_NEWNS)` + обязательный `MS_REC | MS_PRIVATE`-remount `/` перед реальным монтированием (без этого шага mount-события всё равно утекают в хостовую таблицу через shared propagation, унаследованную от родительского namespace), монтирует раздел в `/sysroot`. `deploy(prefix_digest) -> PathBuf` — один компонент пути (`state/deploy/<usr-digest>/`, см. **§3**). `open_repository()` / `open_tree(name)` — единственная публичная точка доступа к composefs-репозиторию текущего деплоя (См. выше);
- `database` — БД пакетов (реализация через redb) внутри образа. При сборке пишется через свой in-memory `StorageBackend`, в runtime читается `ReadOnlyDatabase` с файла в образе. Там же `record` — `DeployRecord`/`EtcHistoryEntry` (Поля см. **§3** п.12: `prefix_digest`/`subject`/`message`/`seq`/`timestamp`/`etc_history`/`working_etc`), физически не часть redb-БД (отдельный `meta.json` на sysroot, не внутри образа), но живёт здесь же по смыслу — *"как наши типы персистятся"* общая забота `database`, независимо от формата. Сериализация — `#[derive(JsonCodec)]` (по образцу `RedbCodec`, тот же по-полевой codegen, только в `serde_json::Value` вместо байт-layout'а); `DeployRecord::write`/`read` пишут/читают файл, `write` — атомарно (tmp-файл в той же директории + `fsync` + `rename`). Своя ошибка `error::DeployRecordError` (отдельно от `DatabaseError` — разные форматы хранения);
- `types` — доменные типы (`Version`, `PackageMeta`, `Dependency`, `Targets`...) и per-commands `StateId`-enums (`states`);
- `errors` / `lock` — вынесены из `types` в свои топ-уровневые публичные модули: `CommonError` (обёртка над `HookError`/`DecoderError`/`RepoError`/`DatabaseError`/`SysrootError`/`LockError`/`DeployRecordError` — все они теперь тоже публичны, каждый под своим модулем выше) и `Lock`/`LockError` (эксклюзивный системный лок файл, смю **§5.9**);

***Ещё не начато:*** `etc_merge` (3-way слияние `/etc`, §5.1), `boot` (загрузочной записи, разовый вход/подтверждение/откат, см. **§5.2**; берёт `composefs-boot`, grub через `blscfg` — отдельных плагинов нет, только BLS-совместимые загрузчики by desing), `gc` (политика удержания и очистки, см. **§5.5**), и само построение графа зависимостей пакетов на стороне `lib` (decoder сейчас только отдаёт сырой список зависимостей пакета через `decode` — граф ещё никто не обходит, да и сетевого слоя для скачивания пакетов тоже нет).

Конфиг времени сборки: имена таблиц, пути деплоя, адрес лока — это `lib.toml` + `build.rs`, генерирующие простые константы, а не отдельный крейт `derive-static` — эта идея заменена. Возможно замена в будущем.

**`cli/`** — тонкая обёртка над библиотекой:

- `args` — разбор аргументов;
- `commands` — по модулю на команду;
- `render` — рендер прогресса, событий и конфликтов из хуков;
- `ffi` — привязка к C-ABI ядра.

Возможны изменения в следвии дальнейшей адапации посте стабилизации кода библиотеки.

**`decoders/`** — плагины (По одному на формат упаковки и сжатия пакета: alpm / deb / rpm / xbps и т.д. Загружает и вызывает их `lib` (модуль `decoder`, внутри родительской папки `plugin`), **НЕ** CLI или другой внешний код. Какой плагин загружать под какой формат — решается по декларативному манифесту (`/etc/upac.d/decoders/*.toml`, см. **§5.8**). По умолчанию — динамические `.so`, опционально мейнтейнеры дистрибутива собирают статикой линковкой.
