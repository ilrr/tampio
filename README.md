# Tampio

Tampio on komentoriviohjelma kirjanpitoraporttien luomiseen, voisipa joku sanoa sitä kirjanpito-ohjelmaksikin. Ohjelman käyttö perustuu tekstimuotoisiin kirjanpitotiedostoihin, joiden pohjalta se luo HTML-muotoisen raportin. Tampio on luotu pääasiassa erään yhdistyksen tarpeita silmällä pitäen, enkä suosittele sitä ainakaan kovin suurimittaiseen käyttöön. Jos kiinnostuit kirjanpidon järjestämisestä tekstitiedostolla, suosittelen tutustumaan kypsempiin *[Plain Text Accounting](https://plaintextaccounting.org/)* -ohjelmiin.

Tampiolle on [Tree-sitter-kielioppi](https://github.com/ilrr/tree-sitter-tampio), jonka voit halutessasi ottaa käyttöön.

## Käyttö

Valmiiksi käännetty Tampio löytyy Linuxille Releases-kohdasta oikealta.
Vaihtoehtoisesti voit ladata Tampion lähdekoodin ja käyttää sitä miten parhaaksi näet.

Raportin luonti onnistuu yksinkeraisimmillaan seuraavasti

```bash
tampio kirjanpito.tamp -o tilinpäätös.html
```

Komento lukee `kirjanpito.tamp`-tiedoston ja luo raportin tiedostoon `tilinpäätös.html`. Tulostetiedoston voi jättää määrittämättä, jolloin raportti tulostetaan suoraan stdoutiin.

Tampiolle voi syöttää useamman tiedoston, jolloin ensimmäiselle luodaan raportti jossa jälkimmäisiä käytetään vertailutietoina.

```bash
tampio kirjanpito2000.tamp talousarvio2000.tamp kirjanpito1999.tamp -o tilinpäätös2000.html
```

Vertailutiedot näkyvät raportissa käänteisessä järjestyksessä kuin komennossa.

Raportin värimaailma on kaunis luonnonläheisen ruskea.

Tarkemmat käyttöohjeet löytyvät [wikin](https://github.com/ilrr/tampio/wiki) puolelta.
