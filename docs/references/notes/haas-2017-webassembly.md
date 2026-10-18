# Haas i in. (2017) – „Bringing the Web up to Speed with WebAssembly”

Notatka do pracy poleconej przez promotora. Liczby i tezy sprawdzono w tekście artykułu ([lokalna kopia](../papers/2017-haas-bringing-the-web-up-to-speed-with-webassembly-pldi.pdf)); cytaty zastąpiono parafrazą.

## 1. Dane bibliograficzne

| | |
|---|---|
| Autorzy | Andreas Haas, Andreas Rossberg, Derek L. Schuff, Ben L. Titzer (Google), Michael Holman (Microsoft), Dan Gohman, Luke Wagner, Alon Zakai (Mozilla), JF Bastien (Apple) |
| Tytuł | Bringing the Web up to Speed with WebAssembly |
| Miejsce | PLDI 2017 – 38th ACM SIGPLAN Conference on Programming Language Design and Implementation, Barcelona, s. 185–200 |
| DOI | [10.1145/3062341.3062363](https://doi.org/10.1145/3062341.3062363) |
| Wersja skrócona | Rossberg A. i in., Communications of the ACM 61(12), 2018, s. 107–115, DOI [10.1145/3282510](https://doi.org/10.1145/3282510) – [kopia](../papers/2018-rossberg-bringing-the-web-up-to-speed-with-webassembly-cacm.pdf) |

```bibtex
@inproceedings{haas2017webassembly,
  author    = {Haas, Andreas and Rossberg, Andreas and Schuff, Derek L. and Titzer, Ben L. and
               Holman, Michael and Gohman, Dan and Wagner, Luke and Zakai, Alon and Bastien, J. F.},
  title     = {Bringing the Web up to Speed with {WebAssembly}},
  booktitle = {Proceedings of the 38th ACM SIGPLAN Conference on Programming Language Design and Implementation},
  series    = {PLDI 2017},
  pages     = {185--200},
  year      = {2017},
  publisher = {ACM},
  doi       = {10.1145/3062341.3062363}
}

@article{rossberg2018webassembly,
  author  = {Rossberg, Andreas and Titzer, Ben L. and Haas, Andreas and Schuff, Derek L. and
             Gohman, Dan and Wagner, Luke and Zakai, Alon and Bastien, J. F. and Holman, Michael},
  title   = {Bringing the Web up to Speed with {WebAssembly}},
  journal = {Communications of the ACM},
  volume  = {61},
  number  = {12},
  pages   = {107--115},
  year    = {2018},
  doi     = {10.1145/3282510}
}
```

## 2. Treść pracy w skrócie

**Problem.** JavaScript stał się celem kompilacji innych języków (Emscripten, asm.js), ale ma nieprzewidywalną wydajność i nie da się go łatwo rozszerzać (np. o 64-bitowe liczby całkowite). Wcześniejsze podejścia – ActiveX, Native Client, asm.js – nie spełniały jednocześnie wymagań bezpieczeństwa, wydajności i przenośności.

**Cele projektowe** (sekcja 1.1): semantyka bezpieczna, szybka, przenośna i deterministyczna, prosta współpraca z platformą WWW; reprezentacja zwarta, łatwa do dekodowania, walidacji i kompilacji (także strumieniowej i równoległej); niezależność od języka, sprzętu i platformy – także poza przeglądarką.

**Najważniejsze elementy projektu:**

- maszyna stosowa z typowanym, ustrukturyzowanym przepływem sterowania (bez dowolnych skoków), walidowana w jednym przebiegu przed wykonaniem (sekcje 2.1, 2.3, 4);
- **pamięć liniowa** (sekcja 2.2) – tablica bajtów powiększana stronami po 64 KiB, oddzielona od kodu, stosu wykonania i struktur silnika; każdy dostęp jest sprawdzany względem rozmiaru, a wyjście poza zakres kończy się pułapką (trap). W najgorszym razie błędny lub zaatakowany moduł psuje tylko własne dane, więc niezaufany kod może działać w tej samej przestrzeni adresowej co reszta strony;
- wywołania pośrednie przez tablicę funkcji z dynamicznym sprawdzeniem sygnatury;
- współpraca z otoczeniem wyłącznie przez importy i eksporty (API JavaScript);
- formalna semantyka (wykonanie i walidacja) z dowodem poprawności systemu typów;
- ograniczony niedeterminizm (sekcja 2.5): tylko wzorce bitowe NaN i wyczerpanie zasobów (np. nieudane `grow_memory`).

**Techniki implementacyjne** (sekcje 7.1–7.2): na platformach 64-bitowych silnik rezerwuje 8 GiB przestrzeni wirtualnej i używa stron ochronnych, dzięki czemu sprawdzanie granic pamięci jest praktycznie darmowe; kompilacja może odbywać się równolegle i strumieniowo, w trakcie pobierania modułu.

**Wyniki** (sekcja 7.3, PolyBenchC, V8 i SpiderMonkey, średnia z 15 przebiegów):

| Pomiar | Wynik |
|---|---|
| WebAssembly vs kod natywny | 7 z 24 programów w granicy 10%, prawie wszystkie poniżej 2× |
| WebAssembly vs asm.js | średnio o 33,7% szybciej; walidacja w SpiderMonkey < 3% czasu walidacji asm.js |
| Rozmiar kodu | 62,5% rozmiaru zminimalizowanego asm.js, 85,3% kodu natywnego x86-64 |
| Stały koszt startu maszyny wirtualnej | 18 ms (V8), 30 ms (SpiderMonkey) |

## 3. Czego praca nie obejmuje

1. **Reprezentatywności obciążenia.** PolyBenchC to małe jądra obliczeniowe (ok. 100 linii, bez wywołań systemowych). Jangda i in. (2019, [W1]) na SPEC CPU zmierzyli średnio 1,45–1,55× i maksymalnie 2,5× wolniejsze wykonanie oraz wskazali przyczyny: więcej odczytów i zapisów pamięci (presja na rejestry), dodatkowe skoki (sprawdzanie przepełnienia stosu przy każdym wywołaniu, kontrola wywołań pośrednich), większy kod.
2. **Kosztu granicy JavaScript ↔ WebAssembly.** Artykuł stawia prostą współpracę z platformą WWW za cel, ale nie mierzy kosztu przekazywania danych przez granicę – a dla silnika bazy danych zwracającego wiersze do JS jest to koszt dominujący (H3 w projekcie).
3. **Późniejszej ewolucji standardu.** Opisano MVP: pamięć 32-bitowa (do 4 GiB), brak wątków, SIMD, GC i wyjątków. Wasm 3.0 (09.2025, [W10]) dodaje m.in. memory64, wiele pamięci, GC i wyjątki, a obok niego JS String Builtins. Memory64 ma swoją cenę: sprawdzanie granic nie może już opierać się na samych stronach ochronnych ([W6]: ponad 100% narzutu przy kontrolach programowych, 12,7% przy dwupoziomowych stronach ochronnych).
4. **Bezpieczeństwa wewnątrz modułu.** Izolacja dotyczy granicy moduł–host. Lehmann i in. (2020, [W2]) pokazali, że w pamięci liniowej brakuje kanarków stosu, ASLR i stron ochronnych, więc błędy z języków niebezpiecznych pamięciowo są wykorzystywalne; Corrias i in. (2026, [W9]) – że takie błędy przekładają się na klasyczne ataki webowe (m.in. SQL injection).

## 4. Odniesienie do Flat-SQL

| Założenie lub wynik projektu | Co mówi praca | Ocena |
|---|---|---|
| „Jeden rdzeń, wiele środowisk” – ten sam kod w przeglądarce, Node.js, na węźle edge i natywnie ([architecture.md §1](../../architecture.md)) | niezależność od języka, sprzętu i platformy jest celem projektowym; przewidziano zastosowania poza przeglądarką | zgodne |
| H2: WASM 1,1–1,8× wolniejszy od natywnego w operacjach obliczeniowych ([benchmark-results.md §6](../../research/benchmark-results.md)) | prawie wszystkie programy poniżej 2× kodu natywnego; Jangda (2019): średnio 1,45–1,55× | wynik projektu mieści się w przedziale z literatury |
| Spowolnienie filtrowania przez dużą ramkę stosu rekurencyjnego `eval` ([benchmark-results.md §5](../../research/benchmark-results.md)) | praca nie analizuje; Jangda (2019) wskazuje presję na rejestry i sprawdzanie stosu przy każdym wywołaniu jako główne źródła narzutu | obserwacja projektu potwierdza diagnozę z [W1] – warto to zaznaczyć w pracy |
| Moduł 545 KiB (206 KiB gzip), kompilacja 2,7 ms w Node.js i 28,8 ms w Chromium | zwarta reprezentacja, szybka walidacja i kompilacja, kompilacja strumieniowa | zgodne; serwer wysyła `.wasm` z typem `application/wasm`, co pozwala na kompilację strumieniową |
| H3: wynik jako JSON 1,9× szybszy niż obiekty JS budowane przez `js-sys` | brak pomiarów granicy JS↔WASM | **wkład własny projektu** – uzupełnia lukę w [P2] |
| Privacy by Design: przetwarzanie danych w „izolowanym środowisku przeglądarki” (README) | izolacja chroni hosta przed modułem, nie dane modułu przed jego błędami ani przed innym kodem tej samej strony | częściowo – zob. [literature-comparison.md §2.4](../literature-comparison.md#24-izolacja-bezpieczeństwo-i-privacy-by-design) |
| Dane w całości w pamięci (`BTreeMap`), scenariusz „CSV 500 MB” w README | pamięć liniowa MVP ma 32-bitowe adresy | ograniczenie: reprezentacja wierszy jako osobnych alokacji zwielokrotnia rozmiar danych; dla takich zbiorów właściwszy jest DuckDB-Wasm ([D1]) |
| Rozgrzewka przed pomiarem (Liftoff → TurboFan, do 2× różnicy przy insertach) | praca opisuje kompilację z 2017 r.; kompilator bazowy Liftoff w V8 pojawił się później | zgodne z rozwojem silników po publikacji |
| Snapshot i ramki WAL kodują `f64` bit po bicie | jedynym niedeterminizmem arytmetyki są wzorce bitowe NaN | drobna uwaga: przy porównywaniu replik po skrócie bajtów warto kanonizować NaN przy zapisie |

## 5. Propozycje sformułowań do pracy (parafrazy)

- WebAssembly zaprojektowano jako przenośny, niskopoziomowy format kodu o formalnie zdefiniowanej semantyce, walidowany przed wykonaniem i izolowany od środowiska wykonawczego przez pamięć liniową [Haas 2017].
- Autorzy standardu raportowali wydajność zbliżoną do natywnej na jądrach obliczeniowych PolyBenchC [Haas 2017], jednak późniejsze pomiary na pełnych aplikacjach SPEC CPU wykazały średnio 45–55% narzutu [Jangda 2019]; zmierzony w niniejszej pracy narzut silnika Flat-SQL (1,1–1,8× dla operacji wewnątrz silnika) mieści się w tym przedziale.
- Izolacja zapewniana przez WebAssembly chroni środowisko hosta, ale nie zabezpiecza danych w pamięci modułu przed błędami samego modułu [Lehmann 2020]; dlatego implementacja silnika w bezpiecznym podzbiorze języka Rust ma znaczenie także dla bezpieczeństwa.
