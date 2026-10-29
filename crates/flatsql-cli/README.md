# flatsql-cli – powłoka i benchmark natywny

Program `flatsql` uruchamia [rdzeń](../flatsql-core/) natywnie: interaktywną powłokę SQL nad bazą w pamięci albo w katalogu oraz natywną wersję obciążenia benchmarku. Baza w katalogu ma ten sam format plików co serwer Node.js, więc dane serwera można otworzyć w CLI.

## Budowanie

```bash
cargo build --release -p flatsql-cli
```

Program trafia do `target/release/flatsql` (w Windows `flatsql.exe`).

## Użycie

```bash
cargo run --release -p flatsql-cli -- moja-baza
```

```text
flatsql> CREATE TABLE notes (id INT PRIMARY KEY, body TEXT);
flatsql> INSERT INTO notes VALUES (1, 'zażółć gęślą jaźń');
flatsql> EXPLAIN SELECT * FROM notes WHERE id = 1;
flatsql> .stats
```

| Wywołanie | Działanie |
|---|---|
| `flatsql [KATALOG]` | otwiera bazę w katalogu (tworzy go, jeśli nie istnieje) albo, bez katalogu, bazę w pamięci |
| `-e SQL` | wykonuje SQL i kończy |
| `-f PLIK` | wykonuje skrypt i kończy |
| `--no-fsync` | nie wykonuje `fsync` po każdym zatwierdzeniu (checkpoint jest synchronizowany zawsze) |
| `flatsql bench [--rows N] [--json PLIK] [--storage KATALOG]` | obciążenie benchmarku natywnie; z `--storage` także trwałe zatwierdzenia na plikach |

Polecenia powłoki: `.tables`, `.schema TABELA`, `.stats`, `.checkpoint`, `.timer on|off`, `.help`, `.quit`. Katalog bazy zawiera `snapshot.fsql` i `wal.fsql` ([formaty](../flatsql-core/src/storage/README.md#formaty-binarne)).

## Struktura

| Plik | Rola |
|---|---|
| `src/main.rs` | powłoka, argumenty, baza w pamięci albo `Persistent<FileStorage>` |
| `src/bench.rs` | to samo obciążenie co [bench/suite.js](../../bench/suite.js), z tą samą sumą kontrolną wyników i identyfikatorem obciążenia |

Benchmark natywny jest częścią mierzonej wersji silnika: runner benchmarku uruchamia `flatsql bench` z `target/release` tej wersji i przerywa pomiar, gdy CLI zgłasza niezgodne obciążenie ([bench/README.md](../../bench/README.md)).
