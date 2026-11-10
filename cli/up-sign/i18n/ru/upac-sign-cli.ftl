error = Ошибка

err-read = Ошибка чтения
err-write = Ошибка записи
err-malformed = Повреждённые PKI-данные
err-invalid-signature = Неверная подпись
err-generation = Ошибка генерации сертификата

signature-valid = Подпись верна

clap-usage = Использование
clap-options = Параметры
clap-arguments = Аргументы
clap-commands = Команды
clap-help = Показать справку
clap-version = Показать версию
clap-help-subcommand = Показать эту справку или справку по указанной подкоманде
clap-error = ошибка
clap-error-try-help = Подробнее: '{ $help }'.
clap-error-missing-argument = не указаны обязательные аргументы: { $arguments }
clap-error-invalid-value = недопустимое значение '{ $value }' для '{ $argument }'
clap-error-unknown-argument = неизвестный аргумент '{ $argument }'
clap-error-unknown-subcommand = неизвестная подкоманда '{ $subcommand }'
clap-error-missing-subcommand = '{ $command }' требует подкоманду
clap-error-conflict = аргумент '{ $argument }' нельзя использовать вместе с '{ $prior }'

about = Подпись и проверка hook-файлов upac сертификатами Ed25519
about-generate-root = Создать самоподписанный корневой сертификат и его закрытый ключ
arg-generate-root-common-name = Общее имя (CN) корневого сертификата
arg-generate-root-key-out = Куда записать закрытый ключ корневого сертификата
arg-generate-root-cert-out = Куда записать корневой сертификат
about-generate-cert = Выпустить сертификат для подписи от корневого сертификата
arg-generate-cert-common-name = Общее имя (CN) нового сертификата
arg-generate-cert-root-key = Закрытый ключ корневого сертификата, которым подписывается сертификат
arg-generate-cert-root-cert = Корневой сертификат, выпускающий сертификат
arg-generate-cert-key-out = Куда записать новый закрытый ключ
arg-generate-cert-cert-out = Куда записать новый сертификат
about-sign-hook = Подписать hook-файл
arg-sign-hook-hook = Hook-файл для подписи
arg-sign-hook-key = Закрытый ключ сертификата для подписи
arg-sign-hook-cert = Сертификат для подписи
arg-sign-hook-signature = Куда записать подпись
about-verify-hook = Проверить подпись hook-файла
arg-verify-hook-hook = Hook-файл для проверки
arg-verify-hook-signature = Файл подписи hook-файла
arg-verify-hook-root-cert = Корневой сертификат, к которому должна восходить цепочка сертификата подписи
