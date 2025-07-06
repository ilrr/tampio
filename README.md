# Tampio

Tampio on komentoriviohjelma kirjanpitoraporttien luomiseen, voisipa joku sanoa sitä kirjanpito-ohjelmaksikin. Ohjelman käyttö perustuu tekstimuotoisiin kirjanpitotiedostoihin, joiden pohjalta se luo HTML-muotoisen raportin. Tampio on luotu pääasiassa erään yhdistyksen tarpeita silmällä pitäen, enkä suosittele sitä ainakaan kovin suurimittaiseen käyttöön. Jos kiinnostuit kirjanpidon järjestämisestä tekstitiedostolla, suosittelen tutustumaan kypsempiin *[Plain Text Accounting](https://plaintextaccounting.org/)* -ohjelmiin.

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

## Tiedostoformaatti

### Yleistä kieliopista

#### Lohkot

Lähes kaikki kirjanpitodata kuvataan erinäisinä *lohkoina*, joilla on *otsake* ja *sisältö*. Sisältö on *erotinmerkeillä* erotettu lista lohkoja tai *atomeja*. Erotinmerkkejä ovat `;` ja rivinvaihto. Lohkon otsake on lista ennaltamääritellynlaisia atomeja, ja se määrittää lohkon merkityksen. Sisältö seuraa otsaketta, ja sen alku ja loppu voidaan määritellä kolmella tavalla. Esimerkeissä alempana voidaan käyttää mitä tahansa näistä tavoista sekaisin.

##### Sulut

Lohkon sisällön voi kääriä sulkeisiin. Mitkä tahansa Unicode-standardin [Bidi-sulkeet](https://www.unicode.org/Public/16.0.0/ucd/BidiBrackets.txt) käyvät, kunhan suljet lohkon avaavaa sulkumerkkiä vastaavalla sulkevalla merkillä.

```
otsake (a; b; c)
9999 0 [1; 3; 10]
⟪"åäö"⟫
```

##### Sisennys

Sisällön voi myös aloittaa sisennyksellä. Töllöin kaikkien lohkon sisällä olevien rivien tulee olla samalla sisennystasolla, ellet aloita uutta lohkoa, jolloin sisennystasoa on luonnollisesti taas nostettava.

```
abc
    1; 2; 3

abc
    1
    2
    3
```

##### Kaksoispiste

Lyhyiden sisältöjen määrittämiseen voi käyttää kaksoispistettä. Kaksoispiste avaa lohkon sisällön ja se suljetaan rivinvaihtoon (tai tiedoston loppuun). Seuraavat lohkot ovat identtisiä

```
abc (1; 2; 3)

abc: 1; 2; 3
```

Kaksoispistelohkoja voi laittaa myös sisäkkäin. Esim. lohkon `1(2(3))` voi kirjoittaa

```
1: 2: 3
```

#### Atomit

##### Luku

Luku on kokonaisluku tai se voi sisältää desimaaliosan. Desimaalierottimena voi käyttää sekä pistettä että pilkkua.

```
1
1942
0
2,56
0.9
```

##### Tunniste

Tunniste alkaa kirjainmerkillä jota voi seurata mikä tahansa määrä kirjaimia, numeroita ja alaviivoja.

```
pankki
Ö__Ä__
H2
風
```

##### Merkkijono

Merkkijonot kääritään lainausmerkkien sisään. Lainausmerkeiksi käyvät `"` ja `'`, mutta myös mitkä tahansa Unicode-kategorioihin `Pi` tai `Pf` kuuluva merkki käy. Avaavan ja sulkevan lainausmerkin on oltava sama. Jos siis haluat käyttää tiettyä lainausmerkkiä merkkijonon osana, kirjoita merkkijono toisenlaisten merkkien sisään. Merkkijono ei voi sisältää rivinvaihtoja.

##### Päivämäärä

Päivämäärä on muotoa `d.m.y`, jossa `d` tarkoittaa päivää, `m` kuukautta ja `y` vuotta. Vuoden voi kirjoittaa kokonaan (`1.1.2025`) tai lyhennettynä (`1.1.25`). Jälkimmäisessä tapauksessa oletetaan päivämäärän olevan 2000-luvulla. Muista kuin dokumentin ensimmäisestä päivämäärästä vuosiluvun voi myös jättää kokonaan pois (`1.1.`), jolloin vuodeksi tulkitaan dokumentissa viimeksi esiintynyt vuosi.

#### Muut lausekkeet

##### Alias

Tilikartan tilille voi luoda aliaksen, joka on muotoa `tilin_nimi = 1234`, eli tunniste, yhtäsuuruusmerkki ja tilin numero. Aliasta voi käyttää tilinumeron sijasta missä tahansa lausekkeessa. Lohkon sisällä määritelty alias on voimassa vain kyseisen lohkon sisällä.

##### Osio

Osio avataan otsikolla. Otsikko koostuu `§`-merkistä, jota seuraa osion nimi. Muutamat nimet on varattu erityisille osioille. Kirjainkoolla ei ole väliä, eli `§ OSIO`, `§osio` ja `§ Osio` tarkoittavat kaikki samaa.

##### Kommentti

Kommentti alkaa kahdella yhdysmerkillä (`--`) ja päättyy rivin päättyessä.


### Tilikartta

Tilikarttaosio alkaa otsikolla `§ TILIKARTTA` ja koostuu listasta päätilien tilimääritelmiä. Tilimääritelmän otsake sisältää tilin numeron, mikäli sellainen on, sekä tilin nimen merkkijonona. Juuritilien otsake voi lisäksi alkaa `+`- tai `-`-merkillä, jolloin tiliä ja sen alatilejä käsitellään joko vastaavaa- tai vastattavaa-tileinä. Muuten tiliä käsitellään tulostilinä. Tilimääritelmän vartalo sisältää tilin alatilit, joilla voi puolestaan olla omia alatilejä.

```
+ "Vastaavaa"
    100 "Pankkitili"
    120 "Säästöpossu"
- "Vastattavaa"
    810 "Velat"
    900 "Edellisten tilikausien tulos"
"Tulos"
    "Varsinainen toiminta"
        200 "Varsinaisen toiminnan menot"
        205 "Varsinaisen toiminnan tulot"
    400 "Avustukset"
        415 "Leijonakerhon avustus"
        425 "Äidin avustus"
        435 "Muut avustukset"
```

### Tilikausi

Varsinainen kirjanpitodata kuvataan tilikausiosiossa. Osion otsikko voi olla mikä tahansa, kunhan se ei ole käytössä muunlaisen osion nimenä. Myös ennen yhdenkään osion aloittamista olevat lausekkeet käsitellään tilikausiosioon kuuluvina. Tilikausi muistuttaa kirjanpidon päiväkirjaa ja koostuu seuraavanlaisista lohkoista.

#### Tilitapahtuma

Tärkein lohkotyyppi on *tilitapahtuma*. Sen otsake sisältää tilitapahtuman päivämäärän ja tilitapahtuman kuvausmerkkijonon. Tilitapahtuman sisältö on lista lohkoja, joiden otsake on jonkin kirjanpitotilin numero ja sisältö kyseiselle tilille tilitapahtumassa kirjatut summat. Positiivinen summa tarkoittaa debetiä ja negatiivinen kreditiä.

```
1.1.1970 "Karkkia"
    100: -2,56
    200:  2,56
```

Vaihtoehtoisesti voit käyttää perinteisempiä *kredit*- ja *debet*-merkintöjä. Sallittuja merkintätapoja ovat kreditille `CR`, `Cr`, `cr.`, `CREDIT`, `KREDIT` ja `C`; debetille vastaavasti `DR`, `Dr`, `dr.`, `DEBIT`, `DEBET` ja `D`. Puolen voi merkitä joko ennen rahasummaa tai sen jälkeen.

```
1.1.1970 "Karkkia"
    100: 2,56 KREDIT
    200: DEBET 2,56
```

Tampio tarkastaa, että kunkin tilitapahtuman kredit- ja debetkirjausten summat täsmäävät. Yhden tilin summan voi myös korvata merkinnällä `AUTO`, jolloin Tampio laskee sen automaattisesti siten, että kirjauksen summat ovat tasapainossa.

```
2.2. "Viikkoraha kolikot"
    425
        -2,00; -0,50
    120
        AUTO
```

Tapahtumalla voi olla myös itse määritetty tositetunnus, jonka täytyy olla samaa muotoa kuin tunnisteen. Tositetunnuksen voi kirjoittaa joko tapahtuman kuvauksen jälkeen tai ennen päivämäärää.

```
1.5. "Laina vappupalloa varten" V1
    100: 12,99
    810: AUTO
```

Itsemääritelty tositenumerointi on kätevää mm. liitettäessä pääkirjanpitoon osakirjanpitoja, joilla on oma numerointijärjestelmänsä.

Jos tunnistetta ei ole määritty, Tampio luo tapahtumalle automaattisen tositenumeron. Jos useammalla tapahtumalla on sama tositetunnus, niille luodaan yksilölliset alaindeksit.

#### Päivämäärälohko

Päivämäärälohkon otsake on yksittäinen päivämäärä. Lohkon sisällä voi tilitapahtumien päivämäärän jättää määrittämättä, jolloin päivämääräksi asetetaan otsakkeen päivämäärä.
```
23.12.
    "Joululahja Matiakselle"
        100: AUTO
        200: 19,49
    "Joululahja Tarmolle"
        100: AUTO
        200: 49,95
```

#### Auto-lohko

Otsakkeella `AUTO nnnn`, jossa `nnnn` on jonkin kirjanpitotilin numero, määritellyssä lohkossa kaikki tapahtumat tasapainotetaan käyttäen otsakkeen tiliä.

```
pankki = 100
23.12. ( AUTO pankki
    "Joululahja Matiakselle"
        200: 19,49
    "Joululahja Tarmolle"
        200: 49,95
)
```

Huomaa myös, että lohkoja voi laittaa sisäkkäin.

### Tiedot-osio

Tiedot-osio alkaa otsikolla `§ TIEDOT`. Osio sisältää erilaisia määritelmiä, joita käytetään raportin luonnissa. Määritelmä on muotoa `avain = "arvo"`. Tällä hetkellä ainoa määritelmä on avaimelle `lyhenne`, jonka arvoa käytetään taseen ja tuloslaskelman otsikoissa nykyisen ja vertailutilikausien yksilöintiin.

### Talousarvio

Talousarvio-osion voi aloittaa joko `§ TALOUSARVIO` tai `§ BUDJETTI` -otsikolla. Osion sisältö on yksinkertainen: lista lohkoja, joiden otsakkeina on tilinumero ja sisältönä kyseisen tilin arvioidut tulot/menot. Tilipahtumista poiketen positiivinen luku on tuloa ja negatiivinen menoa. Voit myös käyttää credit- ja debit-merkintöjä, jotka toimivat odotetulla tavalla. Talousarvio säilötään omaan tiedostoon ja sitä voi käyttää joko vertailu"kirjanpitona" tai luoda sen pohjalta oman raportin mahdollisine vertailukirjanpitoineen.

```
§ budjetti
200: -280
205: 10
415: 0
425: 130
435: 10000
```
