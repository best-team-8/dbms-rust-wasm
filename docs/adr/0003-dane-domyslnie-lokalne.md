# 0003. Dane domyślnie lokalne

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-20

## Kontekst

Projekt obiecuje przetwarzanie danych w przeglądarce bez wysyłania ich na serwer (hasło „Privacy by Design”). Literatura rozdziela tu trzy różne własności. Pierwsza to ochrona przeglądarki przed modułem WebAssembly. Druga to ochrona danych modułu przed jego własnymi błędami. Trzecia to ochrona danych przed innymi ludźmi i programami.

## Decyzja

Dane są przetwarzane i przechowywane lokalnie, a synchronizacja jest opcjonalna. Silnik jest napisany w bezpiecznym Ruście, bez `unsafe` w rdzeniu i w wiązaniach WebAssembly (jedyny wyjątek opisuje decyzja 0014). Szyfrowanie danych na urządzeniu i w drodze do serwera jest kierunkiem rozwoju, a nie cechą obecnej wersji.

## Uzasadnienie

- Zasada „prywatność domyślnie” (Cavoukian, 2009) i ideał prywatności z Kleppmann i in. (2019) są spełnione przez to, że dane nie opuszczają urządzenia bez decyzji aplikacji.
- Izolacja WebAssembly chroni przeglądarkę przed modułem, ale nie chroni danych modułu przed błędami pamięci (Lehmann i in., 2020). Bezpieczny Rust usuwa większość tych błędów.

## Odrzucone warianty

- Opieranie obietnicy na samej izolacji WebAssembly. Nie chroni danych przed innymi skryptami tej samej strony, bo IndexedDB jest dostępna dla całej domeny.
- Przechowywanie kluczy kryptograficznych w bazie, na przykład w portfelu kryptowalut.

## Konsekwencje

- „Privacy by Design” znaczy dziś: dane domyślnie lokalne, ale nieszyfrowane.
- Pełna ochrona wymaga szyfrowania lokalnego, szyfrowania paczek zmian przesyłanych przez serwer i uwierzytelniania. Szyfrowanie paczek jest możliwe, bo do rozstrzygnięcia konfliktu serwer potrzebuje tylko jawnego znacznika czasu i skrótu klucza (Kuessner i in., 2022).

## Historia

- 2026-06-02: założenie z pierwotnego opisu projektu.
- 2026-10-20: rozdzielono izolację modułu od ochrony danych.
