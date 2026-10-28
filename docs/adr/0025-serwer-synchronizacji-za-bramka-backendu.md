# ADR-0025: Serwer synchronizacji za bramką backendu

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-20 |
| Rodzaj | architektura |
| Powiązane | ADR-0002, ADR-0003, ADR-0023, ADR-0024 (zawęża); future 4.1, 4.2, 8.4, 8.6, 9.1, 9.7 |

## Kontekst

Prototypowy serwer synchronizacji przyjmuje żądania od każdego: `/sync/pull` oddaje dane wszystkich klientów, `/sync/push` przyjmuje dowolne zmiany, a `/api/query` wykonuje dowolny SQL. Przestrzenie z ADR-0023 wymagają ustalenia, kim jest klient i do której przestrzeni ma dostęp.

Rozważono, kto odpowiada za tożsamość (konta, hasła, logowanie), kto sprawdza dostęp do przestrzeni i którędy płynie ruch synchronizacji.

## Decyzja

1. **Serwer synchronizacji nie zna użytkowników.** Konta, hasła i logowanie należą do backendu aplikacji.
2. **Serwer działa wyłącznie w sieci wewnętrznej, za bramką backendu.** Frontend wysyła `/sync/*` do backendu aplikacji. Backend sprawdza sesję użytkownika, wybiera przestrzeń i przekazuje żądanie przez HTTP do serwera synchronizacji, nie czytając paczek.
3. **Serwer ufa tylko bramce.** Przestrzeń przychodzi w nagłówku ustawionym przez backend. Żądanie bez wspólnego klucza bramki jest odrzucane, także w sieci wewnętrznej.
4. **Integracja z backendami odbywa się wyłącznie przez HTTP.** Backend w dowolnym języku przekazuje żądania standardowymi narzędziami (reverse proxy), bez wiązań Flat-SQL.
5. **Rozszerzenia na przyszłość, bez terminu:**
   - weryfikacja przepustek JWT przez sam serwer, dla aplikacji bez backendu; w serwerze to drugie źródło tego samego identyfikatora przestrzeni, więc nie wymaga przebudowy;
   - serwer synchronizacji jako biblioteka w procesie backendu, przez interfejs C (9.1) i nakładki dla języków (9.7).

## Uzasadnienie

- **Uwierzytelnianie i tak jest w backendzie.** W zastosowaniach z [use-cases.md](../research/use-cases.md) – pamięć podręczna danych SaaS, zbieranie danych w terenie, kasy, gry ze studiem i kontami graczy – aplikacja ma już backend z kontami. Programista dopisuje jeden endpoint przekazujący żądania.
- **Mniejsza powierzchnia ataku.** Do internetu wystawiony jest tylko backend, który i tak trzeba zabezpieczać. Serwer synchronizacji nie potrzebuje HTTPS, publicznego CORS ani limitów żądań z internetu.
- **Mało pracy po stronie silnika.** Serwer odczytuje przestrzeń z nagłówka i sprawdza klucz bramki. Nie potrzebuje bibliotek JWT ani konfiguracji wydawców przepustek.
- **Uznany wzorzec.** ElectricSQL zaleca to samo rozwiązanie („proxy auth”) dla swojej usługi synchronizacji.
- **Brak wiązań.** Przekazywanie bajtów przez HTTP potrafi każdy język, więc nie trzeba utrzymywać nakładek (ADR-0024).

## Odrzucone alternatywy

- **A. Publiczny serwer sprawdzający JWT jako tryb domyślny** (wzór: PowerSync) – więcej pracy w serwerze i większa powierzchnia ataku. Zostaje jako rozszerzenie (pkt 5).
- **C. Konta wbudowane w serwer synchronizacji** (wzór: Dexie Cloud, Actual Budget) – drugi system kont obok kont aplikacji i pełna odpowiedzialność za hasła, reset i maile.
- **Wspólny sekret do podpisywania przepustek (HS256)** – sekret w dwóch miejscach; wyciek z serwera synchronizacji pozwala podrabiać przepustki.
- **Jeden klucz API aplikacji** – klucz jest widoczny w kodzie strony i nie oddziela użytkowników.
- **Serwer synchronizacji tylko jako biblioteka w backendzie** (jak SQLite) – wymaga nakładek dla każdego języka i przeniesienia logiki serwera do Rust. Przy wielu instancjach backendu wymaga blokad między procesami na tych samych plikach przestrzeni.

## Konsekwencje

- Każda paczka przechodzi przez backend, który dźwiga cały ruch synchronizacji. Przy małej i średniej skali to bez znaczenia.
- Aplikacje bez backendu (aplikacje osobiste, gry indie) muszą postawić bramkę albo poczekać na weryfikację JWT (pkt 5).
- `SyncClient` wysyła żądania pod adres backendu; przy tej samej domenie CORS nie jest potrzebny, a sesja jest w ciasteczku.
- Zadania: future 4.1 (przestrzeń z nagłówka bramki, klucz bramki, nasłuch tylko w sieci wewnętrznej), 4.2 (`/api/query` tylko z bramki), 8.4 (przykład bramki i pokaz w demo), 9.7 (biblioteka w backendzie – rozszerzenie).
- ADR-0024 pozostaje w mocy dla pakietu Pythona i synchronizacji w FastAPI. Serwer synchronizacji w procesie backendu wraca jako odległe rozszerzenie dla wszystkich języków (9.7), nie jako zadanie bieżące.

## Źródła

[use-cases.md](../research/use-cases.md) §1–4; ElectricSQL, [Auth guide](https://electric-sql.com/docs/guides/auth); PowerSync, [authentication](https://docs.powersync.com/configuration/auth/firebase-auth); [Dexie Cloud – authentication](https://dexie.org/cloud/docs/authentication); [Actual Budget – OpenID](https://actualbudget.org/docs/config/oauth-auth).
