# 0020. Granica dryfu zegara: odrzucenie, wstrzymanie synchronizacji i nowe znaczniki

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-23

## Kontekst

Zegar hybrydowy przejmuje znaczniki innych urządzeń (decyzja 0019). Urządzenie z zegarem ustawionym w przyszłość wygrywało więc każdy konflikt, a jego czas rozlewał się przez serwer na wszystkie urządzenia. Kulkarni i in. (2014) zalecają odrzucać znaczniki zbyt daleko przed zegarem odbiorcy. Samo odrzucenie nie mówi jednak, co ma zrobić urządzenie, którego zmiany odrzucono. Porzucenie zmian to utrata pracy offline. Ponawianie to wysyłanie bez końca, bo znacznik się nie zmienia.

## Decyzja

1. Granica wynosi 60 s. Zmiana ze znacznikiem dalej niż 60 s przed zegarem odbiorcy nie jest stosowana i nie przesuwa zegara.
2. Wzorcem jest zegar serwera. Każda odpowiedź synchronizacji podaje czas serwera.
3. Klient, którego zegar odbiega o ponad 60 s albo którego zmiany serwer odrzucił za dryf, wstrzymuje synchronizację. Zmiany zostają w kolejce, a aplikacja dostaje zdarzenie `clock`.
4. Po poprawieniu zegara zmiany z kolejki, których znacznik wykracza poza granicę, dostają nowe znaczniki, a zegar cofa się do najnowszego znacznika, który został. Host od razu robi checkpoint, bo nowe znaczniki nie są zapisywane w WAL.
5. Zmiany pobrane z serwera, które są za daleko w przyszłości, nie przesuwają kursora i są pobierane ponownie.

## Uzasadnienie

- Granica i odrzucenie pochodzą z pracy o zegarze hybrydowym. Wartość 60 s przyjęto bez pomiaru.
- Wstrzymanie zachowuje pracę offline: zmiany czekają jak przy braku sieci i trafiają do wszystkich po poprawie zegara.
- Cofnięcie zegara jest bezpieczne, bo znaczniki spoza granicy zostały odrzucone, więc nikt poza urządzeniem ich nie zna.
- Test synchronizacji w trudnych warunkach pokazał, że samo odrzucenie nie wystarcza: bez wstrzymania i nowych znaczników synchronizacja nigdy się nie kończyła.

## Odrzucone warianty

- Odrzucenie bez cofania zegara. Rozjazd urządzeń jest trwały.
- Usunięcie lokalnych zmian i pobranie stanu serwera. Praca wykonana z błędnym zegarem przepada.
- Korekta zegara czasem serwera. Znaczniki zależałyby od pomiaru przez sieć.

## Konsekwencje

- Urządzenie z błędnym zegarem nie synchronizuje się, dopóki zegar nie zostanie poprawiony.
- Dwa urządzenia mogą różnić się prawie o 120 s, każde w granicy wobec serwera. Zmiana jednego bywa wtedy chwilowo odrzucana przez drugie, najwyżej przez 60 s.

## Historia

- 2026-10-23: granica dryfu, wstrzymanie i nowe znaczniki. Wcześniej zegar przejmował każdy znacznik.
