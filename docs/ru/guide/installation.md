# Установка

## Установщики и пакеты

Рекомендуемый способ установки ntpd-rs - через установщик или менеджер пакетов для вашей системы.

### Linux

Мы рекомендуем пакеты с нашей [страницы релизов](https://github.com/pendulum-project/ntpd-rs/releases). Пакет позаботится о размещении конфигурации в нужном месте и настройке рекомендуемых пользователей и разрешений. Файл конфигурации по умолчанию находится в `/etc/ntpd-rs/ntp.toml`

В debian-подобной системе пакет `.deb` можно установить с помощью

```console
$ sudo dpkg -i /path/to/deb/file.deb
```

В red hat-подобной системе пакет `.rpm` можно установить с помощью

```console
$ sudo rpm -ivh /path/to/rpm/file.rpm
$ sudo systemctl start ntpd-rs
```

### FreeBSD

Бинарный файл ntpd-rs доступен в [ports](https://www.freshports.org/net/ntpd-rs/). Файл конфигурации по умолчанию находится в `%%ETCDIR%%/ntp.toml`, что обычно соответствует `/usr/local/etc/ntpd-rs/ntp.toml`.

### macOS

В настоящее время нет пакета или установщика для macOS.

## Установка из исходного кода

На платформах без установщика или пакета сборка из исходного кода является вариантом.
ntpd-rs написан на rust. Мы настоятельно рекомендуем использовать [rustup] для установки
rust toolchain, потому что версия, предоставляемая системными менеджерами пакетов, обычно
устаревает. Обязательно используйте недавнюю версию компилятора rust. Для сборки
ntpd-rs выполните

```sh
cargo build --release
```

Это создает бинарный файл `ntp-daemon` в `target/release/ntp-daemon`, который является
основным NTP-демоном. Запуск его из командной строки для тестирования должен предоставить вывод вроде:

```
> sudo target/release/ntp-daemon -c pkg/common/ntp.toml.default
2023-09-04T12:01:44.055104Z  WARN ntpd::daemon::observer: Abnormal termination of the state observer: Could not create observe socket at "/run/ntpd-rs/observe" because its parent directory does not exist
2023-09-04T12:01:44.055183Z  WARN ntpd::daemon::observer: The state observer will not be available
2023-09-04T12:01:44.071353Z  INFO ntpd::daemon::system: new source source_id=SourceId(1) addr=185.172.91.110:123 spawner=SpawnerId(1)
2023-09-04T12:01:44.071735Z  INFO ntpd::daemon::system: new source source_id=SourceId(2) addr=162.159.200.1:123 spawner=SpawnerId(1)
2023-09-04T12:01:44.071944Z  INFO ntpd::daemon::system: new source source_id=SourceId(3) addr=45.138.55.62:123 spawner=SpawnerId(1)
2023-09-04T12:01:44.072150Z  INFO ntpd::daemon::system: new source source_id=SourceId(4) addr=213.154.236.182:123 spawner=SpawnerId(1)
2023-09-04T12:01:44.084626Z  INFO ntp_proto::algorithm::kalman: No consensus cluster found
2023-09-04T12:01:44.085422Z  INFO ntp_proto::algorithm::kalman: No consensus cluster found
2023-09-04T12:01:44.086879Z  INFO ntp_proto::algorithm::kalman: Offset: 2.3686082232975885+-72.6249392570874ms, frequency: 0+-5773502.691896258ppm
2023-09-04T12:01:44.087846Z  INFO ntp_proto::algorithm::kalman: Offset: 2.7204471636925773+-61.339759726948046ms, frequency: 0+-5000000.000000001ppm
```

Чтобы использовать этот бинарный файл как системный NTP-демон, вы должны также:

- переместить бинарный файл `ntp-daemon` в подходящее место (например, `/usr/bin`),
- настроить конфигурацию в `/etc/ntpd-rs/ntp.toml` (мы рекомендуем скопировать конфигурацию из `docs/examples/ntp.toml.default`),
- настроить разрешения для бинарного файла и файла конфигурации, чтобы бинарный файл мог запускаться и читать конфигурацию,
- настроить бинарный файл как системную службу.

### Запуск как системной службы

Самый простой способ - позволить вашей операционной системе и стандартным инструментам позаботиться о деталях, таких как:

- убедиться, что не запущен конкурирующий NTP-демон
- убедиться, что демон запускается при старте системы
- обработка журналов ntpd-rs

Ниже приведены конфигурации для Linux (с использованием `SystemD`) и FreeBSD (с использованием .rc файла).

#### Linux + SystemD

Это конфигурация SystemD, используемая установщиком ntpd-rs для Linux.

```ini
[Unit]
Description=Rust Network Time Service
Documentation=https://github.com/pendulum-project/ntpd-rs
After=network-online.target
Wants=network-online.target
Conflicts=systemd-timesyncd.service ntp.service chrony.service

[Service]
Type=simple
Restart=no
ExecStart=/usr/local/bin/ntp-daemon
Environment="RUST_LOG=info"
RuntimeDirectory=ntpd-rs
User=ntpd-rs
Group=ntpd-rs
AmbientCapabilities=CAP_SYS_TIME
# Примечание: при запуске сервера на порту по умолчанию 123 необходимы разрешения для привязки к
# низким (<1024) портам, которые можно предоставить с помощью
# AmbientCapabilities=CAP_SYS_TIME CAP_NET_BIND_SERVICE

[Install]
WantedBy=multi-user.target
```

Обратите внимание, что в системе должен присутствовать пользователь ntpd-rs, которого можно создать с помощью

```sh
sudo adduser --system --home /var/lib/ntpd-rs/ --group ntpd-rs
```

или если ваша система не имеет adduser

```sh
sudo useradd --home-dir /var/lib/ntpd-rs --system --create-home --user-group ntpd-rs
```

Этот пользователь должен иметь доступ к папке конфигурации:

```sh
sudo chown ntpd-rs:ntpd-rs /etc/ntpd-rs/ntp.toml
sudo chmod 0644 /etc/ntpd-rs/ntp.toml
```

#### FreeBSD

Это [скрипт rc](https://github.com/freebsd/freebsd-ports/blob/main/net/ntpd-rs/files/ntp_daemon.in), используемый [пакетом ntpd-rs на freshports](https://www.freshports.org/net/ntpd-rs/).

```sh
#!/bin/sh

# PROVIDE: ntp_daemon
# REQUIRE: DAEMON FILESYSTEMS devfs
# BEFORE:  LOGIN
# KEYWORD: nojail resume shutdown

. /etc/rc.subr

name=ntp_daemon
rcvar=ntp_daemon_enable

load_rc_config $name

ntp_daemon_enable=${ntp_daemon_enable-"NO"}
ntp_daemon_config=${ntp_daemon_config-"%%ETCDIR%%/ntp.toml"}
ntp_daemon_socket=${ntp_daemon_socket-"/var/run/ntpd-rs"}

command="/usr/bin/true"
procname="/usr/sbin/daemon"
pidfile="/var/run/${name}.pid"

start_cmd="ntp_daemon_start"
stop_cmd="ntp_daemon_stop"

is_process_running()
{
	[ -f ${pidfile} ] && procstat $(cat ${pidfile}) >/dev/null 2>&1
}

ntp_daemon_start()
{
	[ -d "${ntp_daemon_socket}" ] || /bin/mkdir "${ntp_daemon_socket}"
	/usr/sbin/chown _ntp:_ntp "${ntp_daemon_socket}"
	/usr/sbin/daemon -P ${pidfile} -r -f -o /var/log/ntp_daemon.log -H %%PREFIX%%/bin/ntp-daemon --config "${ntp_daemon_config}"

	if is_process_running; then
		echo "Started ntp-daemon (pid=$(cat ${pidfile}))"
	else
		echo "Failed to start ntp-daemon"
	fi
}

ntp_daemon_stop()
{
	if is_process_running; then
		/bin/rm -rf "${ntp_daemon_socket}"
		local pid=$(cat ${pidfile})
		echo "Stopping ntp-daemon (pid=${pid})"
		kill -- -${pid}
	else
		echo "ntp-daemon isn't running"
	fi
}

run_rc_command "$1"
```

[rustup]: https://rustup.rs