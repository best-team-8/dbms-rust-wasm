# ADR-0003: Privacy by Design – dane domyślnie lokalne

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-06-02 (README); zakres doprecyzowany 2026-10-18 |
| Rodzaj | założenie |
| Powiązane | ADR-0001, ADR-0002; todo 0.1; future 4.1–4.4 |

## Kontekst

README obiecuje „przetwarzanie wrażliwych danych w izolowanym środowisku przeglądarki bez wysyłania ich na serwer”. Literatura rozdziela trzy własności: izolację hosta od modułu, ochronę danych modułu przed jego błędami i ochronę danych przed innymi ([P2], [W2], [V1]).

## Decyzja

Dane są domyślnie przetwarzane i przechowywane lokalnie, a synchronizacja jest opcjonalna. Silnik powstaje w bezpiecznym Ruście (bez `unsafe` w rdzeniu i `flatsql-wasm`). Ochrona kryptograficzna (szyfrowanie lokalne i end-to-end) jest kierunkiem rozwoju, nie cechą prototypu.

## Uzasadnienie

- Zasada „prywatność domyślnie” z [V1] i ideał 6 z [L1] są spełnione przez minimalizację przesyłanych danych.
- Izolacja WASM chroni hosta przed modułem, ale nie dane modułu przed błędami pamięci [W2]; bezpieczny Rust eliminuje większość tych klas błędów.

## Odrzucone alternatywy

- **Opieranie obietnicy na samej piaskownicy WASM** – piaskownica nie chroni danych przed innymi skryptami tej samej strony (IndexedDB jest dostępna dla całej domeny).
- **Przechowywanie kluczy kryptograficznych w bazie** – odrzucone dla portfeli kryptowalut ([use-cases.md](../research/use-cases.md)).

## Konsekwencje

- Teza „Privacy by Design” w pracy oznacza dziś „dane domyślnie lokalne, bez szyfrowania” (todo 0.1).
- Pełna realizacja wymaga szyfrowania end-to-end paczek (serwer jako przekaźnik szyfrogramu – możliwe przy LWW, bo do rozstrzygnięcia konfliktu wystarczy jawny znacznik i skrót klucza [L11]), szyfrowania lokalnego i uwierzytelniania (future 4.1–4.4).

## Źródła

[V1], [L1], [L11], [P2], [W2].
