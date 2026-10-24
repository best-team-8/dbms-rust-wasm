# Wyniki optymalizacji silnika

Dokument opisuje optymalizacje Flat-SQL wykonane po Benchmarku 1. Każdy opis wychodzi od słabej strony prototypu ([benchmark-results.md §1.11](benchmark-results.md#111-słabe-strony-prototypu-i-hipotezy-przyczyn)) i jej przyczyny wskazanej przez rozbicie kosztów ([§1.12](benchmark-results.md#112-rozbicie-kosztów-silnika-pomiar-pomocniczy)), profil albo plan zapytania, a kończy się pomiarem przed zmianą i po niej. Pełny pomiar wersji końcowej razem z prototypem, według [metodyki](benchmark-scenario.md), jest w [benchmark-results.md](benchmark-results.md); testy poprawności zmienionego kodu opisuje [tests-results.md](tests-results.md).

## Opis optymalizacji

| Element | Zawartość |
|---|---|
| Słabość | objaw z Benchmarku 1: operacja, wartość, porównanie z innym silnikiem albo z wykonaniem natywnym |
| Hipoteza | przyczyna i jej dowód: rozbicie kosztów, profil, plan zapytania, analiza kodu |
| Zmiana | co zmieniono, jakie rozwiązania odrzucono i dlaczego, z literaturą |
| Pomiar | polecenia, wersja przed i po (commit przez temat), środowisko, liczba uruchomień, obciążenie procesora przed pomiarem |
| Wynik | wartości przed i po w operacji docelowej oraz wielkości niezależne od obciążenia maszyny: liczba alokacji, rozmiar snapshotu, liczba odczytanych wierszy |
| Wpływ na inne operacje | zmiana pozostałych operacji obciążenia, rozmiaru modułu, czasu startu i pamięci |
| Ograniczenia | czego pomiar nie pokazuje i co go zaburza |

Status: *potwierdzona* (zmiana usunęła wskazany koszt w przewidywanym zakresie), *częściowo* (zysk mniejszy od przewidywanego, z podaną przyczyną) albo *odrzucona* (pomiar nie wykazał zysku albo zmiana pogorszyła inną operację). Optymalizacja odrzucona zostaje w dokumencie razem z pomiarem.

Pomiar przed i po wykonuje się w tej samej sesji, wersja po wersji, bez innych aplikacji obciążających procesor (scenariusz, §2). Zmianę uznaje się za wykazaną według kryterium ze scenariusza (§5.4); szybki pomiar z kilkoma uruchomieniami podaje rozrzut. Opis odwołuje się do literatury identyfikatorami z [indeksu](../references/README.md), do kodu przez plik i funkcję, a nie odwołuje się do rejestru decyzji ani list zadań.

## Zestawienie

| Optymalizacja | Słabość | Operacja docelowa: przed → po | Status |
|---|---|---|---|
