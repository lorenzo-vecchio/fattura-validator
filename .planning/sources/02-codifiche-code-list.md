# Fatture Elettroniche italiane — Parte 2/3

## Codifiche complete (code list ufficiali)

**Data:** 17 settembre 2026 · **Fonte primaria:** enumerazioni dell'XSD ufficiale `Schema_VFPR12_v1.2.3.xsd` lette il 17/09/2026 (descrizioni **verbatim in italiano**) + Allegato A — Specifiche tecniche 1.9.1 + istruzioni modello CU per le causali di pagamento.  
**Legenda:** **[V]** descrizione letta nella fonte primaria · **[S]** fonte secondaria · **[?]** da verificare

> ⚠️ **Avvertenza generale sulle tabelle.** Le code list **non sono stabili nel tempo**. Ogni tabella esiste in versioni diverse a seconda della data di emissione della fattura. Un validatore corretto deve associare a ciascuna tabella una **data di decorrenza** e selezionare il set giusto in base alla data del documento (o, per il controllo di conformità al tracciato corrente, alla data di trasmissione). I punti di rottura noti sono indicati tabella per tabella.

* * *

## 1. `FormatoTrasmissione` (+ attributo `versione`)

| Codice | Descrizione ufficiale | Dove ammesso |
| --- | --- | --- |
| `FPA12` | Fattura verso PA | schema `Schema_VFPA12_V1.2.3.xsd` (namespace `…/fatture/v1.2`) |
| `FPR12` | Fattura verso privati | schema `Schema_VFPR12_v1.2.3.xsd` (namespace `…/fatture/v1.2`) |
| `FSM10` | Fattura verso privati semplificata | schema `Schema VFSM10` (namespace `…/fatture/v1.0`) |

Regole:

*   L'attributo `versione` sulla radice deve essere **uguale** a `FormatoTrasmissione` (**controllo 00428**) [V].
    
*   I due XSD 1.2.3 per PA e privati **differiscono solo** per il valore ammesso qui [V].
    
*   `CodiceDestinatario`: **6 caratteri** con `FPA12`, **7 caratteri** con `FPR12`/`FSM10` (**controllo 00427**) [V].
    
*   Non esiste altro valore. In particolare **non** esistono più i vecchi formati `SDI11`/`SDI12` [S].
    

* * *

## 2. `TipoDocumento` (4 caratteri)

### 2.1 Fattura ordinaria / FPA — enumerazione XSD 1.2.3 [V]

| Codice | Descrizione ufficiale |
| --- | --- |
| `TD01` | Fattura |
| `TD02` | Acconto / anticipo su fattura |
| `TD03` | Acconto / anticipo su parcella |
| `TD04` | Nota di credito |
| `TD05` | Nota di debito |
| `TD06` | Parcella |
| `TD16` | Integrazione fattura reverse charge interno |
| `TD17` | Integrazione/autofattura per acquisto servizi dall'estero |
| `TD18` | Integrazione per acquisto di beni intracomunitari |
| `TD19` | Integrazione/autofattura per acquisto di beni ex art.17 c.2 DPR 633/72 |
| `TD20` | Autofattura per regolarizzazione e integrazione delle fatture (ex art. 6 c.9-bis d.lgs. 471/97 o art.46 c.5 D.L. 331/93) |
| `TD21` | Autofattura per splafonamento |
| `TD22` | Estrazione beni da Deposito IVA |
| `TD23` | Estrazione beni da Deposito IVA con versamento dell'IVA |
| `TD24` | Fattura differita di cui all'art.21, comma 4, terzo periodo lett. a) DPR 633/72 |
| `TD25` | Fattura differita di cui all'art.21, comma 4, terzo periodo lett. b) DPR 633/72 |
| `TD26` | Cessione di beni ammortizzabili e per passaggi interni (ex art.36 DPR 633/72) |
| `TD27` | Fattura per autoconsumo o per cessioni gratuite senza rivalsa |
| `TD28` | Acquisti da San Marino con IVA (fattura cartacea) |
| `TD29` | Comunicazione per omessa o irregolare fatturazione da parte del cedente/prestatore italiano (art. 6, comma 8, D.Lgs. 471/97) |

### 2.2 Fattura semplificata (FSM10) [S]

| Codice | Descrizione |
| --- | --- |
| `TD07` | Fattura semplificata |
| `TD08` | Nota di credito semplificata |
| `TD09` | Nota di debito semplificata |

### 2.3 Codici che NON appartengono a FatturaPA

`TD10` (fattura di acquisto intracomunitario beni), `TD11` (fattura di acquisto intracomunitario servizi), `TD12` (documento riepilogativo art. 6 DPR 695/1996) appartengono al **vecchio tracciato "dati fattura"/esterometro**, nonal formato FatturaPA. Dal 01/07/2022 le operazioni transfrontaliere si comunicano **con il tracciato ordinario** (`TD17`/`TD18`/`TD19` per gli acquisti, `XXXXXXX` per le cessioni). Non accettarli in un validatore FatturaPA.

### 2.4 Vincoli per `TipoDocumento` (controlli SdI — dettaglio in Parte 3/3)

| Codice | Regola (versione corrente, post-1.9/1.9.1) |
| --- | --- |
| **00471** | Cedente ≠ Cessionario obbligatorio per: TD01, TD02, TD03, TD06, TD07, TD16, TD17, TD18, TD19, TD20, TD24, TD25, TD28, TD29 |
| **00472** | Cedente = Cessionario obbligatorio per: **TD21** e **TD27** |
| **00473** | `IdPaese` del cedente vincolato: ≠ `IT` per TD17/TD18/TD19; **=** `SM` per TD28; **=** `IT` per TD29 (versione 1.9/1.9.1) |
| **00474** | TD21: nessuna linea con `AliquotaIVA` = 0 |
| **00475** | `IdFiscaleIVA` del cessionario obbligatorio per: TD16, TD17, TD18, TD19, TD20, TD22, TD23, TD28, TD29 |
| **00476** | `IdPaese` cedente e cessionario **non** possono essere entrambi ≠ `IT` |
| **Unicità** | per TD16–TD20, TD22, TD23, TD28, TD29 il controllo di duplicazione è in capo al **cessionario/committente** |
| **Eccezione TD04** | l'unicità ammette due documenti con stesso cedente+anno+numero **solo se uno dei due è TD04** |

Versioni: TD16–TD27 introdotti dalla **1.6** (04/05/2020); **TD28** dalla 1.7.1 (01/08/2022, esteso in 1.8); **TD29** dalla **1.9** (01/04/2025).

* * *

## 3. `Natura` (2–4 caratteri)

### 3.1 Elenco completo (ordinaria) — descrizioni verbatim dall'XSD [V]

| Codice | Descrizione ufficiale | Stato |
| --- | --- | --- |
| `N1` | Escluse ex. art. 15 del D.P.R. 633/1972 | valido |
| `N2` | Non soggette | **DEPRECATO** (fatture dal 01/01/2021) |
| `N2.1` | Non soggette ad IVA ai sensi degli artt. da 7 a 7-septies del DPR 633/72 | valido |
| `N2.2` | Non soggette – altri casi | valido |
| `N3` | Non imponibili | **DEPRECATO** (dal 01/01/2021) |
| `N3.1` | Non Imponibili – esportazioni | valido |
| `N3.2` | Non Imponibili – cessioni intracomunitarie | valido |
| `N3.3` | Non Imponibili – cessioni verso San Marino | valido |
| `N3.4` | Non Imponibili – operazioni assimilate alle cessioni all'esportazione | valido |
| `N3.5` | Non Imponibili – a seguito di dichiarazioni d'intento | valido |
| `N3.6` | Non Imponibili – altre operazioni che non concorrono alla formazione del plafond | valido |
| `N4` | Esenti | valido |
| `N5` | Regime del margine/IVA non esposta in fattura | valido |
| `N6` | Inversione contabile (per le operazioni in reverse charge ovvero nei casi di autofatturazione per acquisti extra UE di servizi ovvero per importazioni di beni nei soli casi previsti) | **DEPRECATO** (dal 01/01/2021) |
| `N6.1` | Inversione contabile – cessione di rottami e altri materiali di recupero | valido |
| `N6.2` | Inversione contabile – cessione di oro e argento ai sensi della legge 7/2000 nonché di oreficeria usata ad OPO | valido |
| `N6.3` | Inversione contabile – subappalto nel settore edile | valido |
| `N6.4` | Inversione contabile – cessione di fabbricati | valido |
| `N6.5` | Inversione contabile – cessione di telefoni cellulari | valido |
| `N6.6` | Inversione contabile – cessione di prodotti elettronici | valido |
| `N6.7` | Inversione contabile – prestazioni comparto edile e settori connessi | valido |
| `N6.8` | Inversione contabile – operazioni settore energetico | valido |
| `N6.9` | Inversione contabile – altri casi | valido |
| `N7` | IVA assolta in altro stato UE (prestazione di servizi di telecomunicazioni, tele-radiodiffusione ed elettronici ex art. 7-octies lett. a, b, art. 74-sexies DPR 633/72) | valido |

**Note critiche:**

*   **Non esistono** `N7.x` **né** `N8`**.** Al 1.9.1 nessuna nuova Natura è stata introdotta [V/S].
    
*   I codici **generici** `N2`, `N3`, `N6` sono ancora **enumerati nell'XSD** (per retro-compatibilità, con commento XML che ne segnala la non validità dal 01/01/2021), ma il SdI li scarta con **00445**. Un validatore deve: accettarli sintatticamente (XSD) e **segnalarli come errore di contenuto**.
    
*   `N3.5` (dichiarazioni d'intento) è legato al **00477**: se la dichiarazione d'intento è invalidata, il file viene scartato — ma la verifica richiede la banca dati AdE (non offline).
    
*   `N5` copre anche il "regime del margine" e i casi di IVA non esposta (es. agenzie di viaggio).
    

### 3.2 Natura ammesse nella **fattura semplificata** (FSM10) [S]

Solo: `N1`, `N2`, `N2.1`, `N2.2`, `N3`, `N3.1`, `N3.2`, `N3.3`, `N3.4`, `N3.5`, `N3.6`, `N4`, `N5`.  
**Escluse** `N6.x` **e** `N7` (l'inversione contabile e l'IVA assolta in altro Stato UE non sono rappresentabili con la fattura semplificata).

### 3.3 Regoelegamento Natura ↔ AliquotaIVA (il cuore della coerenza)

| Dove | Aliquota = 0 e Natura assente | Natura presente e Aliquota ≠ 0 | Codice |
| --- | --- | --- | --- |
| `DettaglioLinee` | errore | errore | **00400 / 00401** |
| `DatiCassaPrevidenziale` | errore | errore | **00413 / 00414** |
| `DatiRiepilogo` | errore | errore | **00429 / 00430** |

**Eccezione TD16**: _"Nel caso in cui l'elemento TipoDocumento assume il valore TD16, è possibile indicare sia la natura delle operazioni, sia un valore diverso da zero per l'elemento AliquotaIVA"_ [V]. → La regola "Natura ⇒ Aliquota 0" **non** va applicata al TD16.
Altre regole: `N6.x` incompatibile con `EsigibilitaIVA = S` (**00420**); ogni Natura usata in linee/cassa deve comparire in un `DatiRiepilogo` (**00444**); ogni `AliquotaIVA` distinta deve avere il suo `DatiRiepilogo` (**00419/00443**).

* * *

## 4. `RegimeFiscale` (4 caratteri) [V]

| Codice | Descrizione ufficiale |
| --- | --- |
| `RF01` | Regime ordinario |
| `RF02` | Regime dei contribuenti minimi (art. 1,c.96-117, L. 244/2007) |
| `RF03` | _Nuove iniziative produttive (art.13, L. 388/00)_ — **RIMOSSO** [S: eliminato il 15/03/2017] |
| `RF04` | Agricoltura e attività connesse e pesca (artt. 34 e 34-bis, D.P.R. 633/1972) |
| `RF05` | Vendita sali e tabacchi (art. 74, c.1, D.P.R. 633/1972) |
| `RF06` | Commercio dei fiammiferi (art. 74, c.1, D.P.R. 633/1972) |
| `RF07` | Editoria (art. 74, c.1, D.P.R. 633/1972) |
| `RF08` | Gestione di servizi di telefonia pubblica (art. 74, c.1, D.P.R. 633/1972) |
| `RF09` | Rivendita di documenti di trasporto pubblico e di sosta (art. 74, c. 1, D.P.R. 633/1972) |
| `RF10` | Intrattenimenti, giochi e altre attività di cui alla tariffa allegata al D.P.R. 640/72 (art. 74, c.6, D.P.R. 633/1972) |
| `RF11` | Agenzie di viaggi e turismo (art. 74-ter, D.P.R. 633/1972) |
| `RF12` | Agriturismo (art. 5, c.2, L. 413/1991) |
| `RF13` | Vendite a domicilio (art. 25-bis, c.6, D.P.R. 600/1973) |
| `RF14` | Rivendita di beni usati, di oggetti d'arte, d'antiquariato o da collezione (art. 36, D.L. 41/1995) |
| `RF15` | Agenzie di vendite all'asta di oggetti d'arte, antiquariato o da collezione (art. 40-bis, D.L. 41/1995) |
| `RF16` | IVA per cassa P.A. (art. 6, c.5, D.P.R. 633/1972) |
| `RF17` | IVA per cassa (art. 32-bis, D.L. 83/2012) |
| `RF18` | Altro |
| `RF19` | Regime forfettario |
| `RF20` | **Regime transfrontaliero di Franchigia IVA (Direttiva UE 2020/285)** — introdotto dalla **1.9** (01/04/2025) |

**Trappole:**

*   L'ordine di enumerazione nell'XSD è `… RF16, RF17, RF19, RF18, RF20` (RF19 **prima** di RF18): non assumere ordinamento.
    
*   `RF03` è stato **rimosso**: un file che lo contiene va segnalato (nell'implementazione di riferimento .NET esiste il codice **00459** per questo caso [S]).
    
*   `RF20` è presente nel file XSD pubblicato **1.2.3** corrente ma **assente** in alcune copie vendored ["1.2.3" del 2025]: **scaricare l'XSD ufficiale** prima di hard-codare l'elenco.
    
*   `RegimeFiscale` è **obbligatorio** (`1..1`) in `CedentePrestatore/DatiAnagrafici`.
    

* * *

## 5. `TipoRitenuta` (4 caratteri) [V]

| Codice | Descrizione ufficiale |
| --- | --- |
| `RT01` | Ritenuta di acconto persone fisiche |
| `RT02` | Ritenuta di acconto persone giuridiche |
| `RT03` | Contributo INPS |
| `RT04` | Contributo ENASARCO |
| `RT05` | Contributo ENPAM |
| `RT06` | Altro contributo previdenziale |

`RT03–RT06` aggiunti dalla **1.6**; `DatiRitenuta` è **0..N** dalla 1.6 (prima 0..1).

* * *

## 6. `CausalePagamento` (max 2 caratteri)

⚠️ **Questa è la tabella più insidiosa**: l'XSD enumera i codici **senza descrizione**, rimandando al modello **CU**. Il riferimento ufficiale è _"i valori ammessi sono quelli della CU consultabili alla pagina delle istruzioni di compilazione del modello"_. Le descrizioni vanno fuse da due fonti.

### 6.1 Valori **enumerati nell'XSD** (verbatim) [V]

```
A  B  C  D  E  G  H  I  L  M  N  O  P  Q  R  S  T  U  V  W  X  Y  Z  L1  M1  M2  O1  V1  ZO
```

*   **Non esistono** `F`, `J`, `K`, `V2` in questa enumerazione.
    
*   Il commento XSD su `Z` recita: _"IL CODICE SEGUENTE (Z) NON SARA' PIU' VALIDO PER LE FATTURE EMESSE A PARTIRE DAL PRIMO GENNAIO 2021"_ → sostituito da `ZO`.
    
*   `X` e `Y` esistono solo nell'XSD e sono **legacy** (rimborsi ritenuta su canoni 2004–2005, procedura del 2006): non hanno voce nella tabella CU corrente.
    

### 6.2 Descrizioni (dal modello CU 2026, periodo d'imposta 2025) [S]

| Codice | Descrizione (sintesi) |
| --- | --- |
| A | prestazioni di lavoro autonomo rientranti nell'esercizio di arte o professione abituale |
| B | utilizzazione economica di opere dell'ingegno, brevetti, processi, formule e informazioni (autore/inventore) |
| C | utili da contratti di associazione in partecipazione/cointeressenza con apporto di solo lavoro |
| D | utili spettanti ai soci promotori e fondatori di società di capitali |
| E | levata di protesti cambiari da parte dei segretari comunali |
| **F** | indennità ai giudici onorari di pace e vice procuratori onorari — **solo CU, NON ammesso in fattura** |
| G | indennità per cessazione di attività sportiva professionale |
| H | indennità per cessazione di rapporti di agenzia (persone fisiche e società di persone) |
| I | indennità per cessazione da funzioni notarili |
| **J** | compensi ai raccoglitori occasionali di tartufi — **solo CU, NON ammesso in fattura** |
| **K** | assegni di servizio civile universale (art. 16 D.Lgs. 40/2017) — **solo CU, NON ammesso in fattura** |
| L | redditi da utilizzazione economica di opere dell'ingegno ecc. percepiti dagli aventi causa a titolo gratuito |
| L1 | come L, ma percepiti da chi ha acquistato a titolo oneroso i diritti |
| M | prestazioni di lavoro autonomo non esercitate abitualmente |
| M1 | redditi da assunzione di obblighi di fare, di non fare o permettere |
| M2 | prestazioni di lavoro autonomo non abituale con obbligo di iscrizione alla Gestione Separata ENPAPI |
| N | indennità di trasferta, rimborsi forfetari, premi e compensi a direttori artistici/collaboratori tecnici (cori, bande, filodrammatiche) |
| O | prestazioni di lavoro autonomo non abituale senza obbligo di iscrizione alla gestione separata ma con obbligo ex Pals |
| O1 | redditi da obblighi di fare/non fare/permettere, con obbligo ex Pals |
| P | compensi a non residenti senza stabile organizzazione per uso di attrezzature, ovvero a società svizzere con i requisiti dell'art. 15 c.2 dell'Accordo 26/10/2004 |
| Q | provvigioni ad agente/rappresentante **monomandatario** |
| R | provvigioni ad agente/rappresentante **plurimandatario** |
| S | provvigioni a commissionario |
| T | provvigioni a mediatore |
| U | provvigioni a procacciatore di affari |
| V | provvigioni a incaricato per vendite a domicilio; vendita porta a porta; vendita ambulante di giornali (L. 67/1987) |
| V1 | redditi da attività commerciali non esercitate abitualmente (provvigioni per prestazioni occasionali) |
| **V2** | redditi da prestazioni non abituali degli incaricati alla vendita diretta a domicilio — **solo CU, NON ammesso in fattura** |
| W | corrispettivi per prestazioni relative a contratti d'appalto con applicazione dell'art. 25-ter DPR 600/1973 |
| X | _(legacy)_ canoni corrisposti nel 2004 con rimborso ritenuta nel 2006 |
| Y | _(legacy)_ canoni corrisposti 01/01/2005–26/07/2005 con rimborso ritenuta nel 2006 |
| Z | _titolo diverso dai precedenti_ — **non più valido per fatture dal 01/01/2021** |
| ZO | titolo diverso dai precedenti (sostituisce `Z`) |

**Conseguenze per il validatore:** se si usa l'enumerazione XSD si è _sintatticamente_ corretti ma si accettano `Z`/`X`/`Y` che il SdI rifiuta (o non usa); se si usa la tabella CU si rischiano falsi errori su codici ammessi dallo XSD. Serve un **set con effective-date**, e un warning separato per i codici "ammessi dallo schema ma non utilizzabili".

* * *

## 7. `TipoCassa` (4 caratteri) — descrizioni verbatim dall'XSD [V]

| Codice | Descrizione ufficiale |
| --- | --- |
| `TC01` | Cassa nazionale previdenza e assistenza avvocati e procuratori legali |
| `TC02` | Cassa previdenza dottori commercialisti |
| `TC03` | Cassa previdenza e assistenza geometri |
| `TC04` | Cassa nazionale previdenza e assistenza ingegneri e architetti liberi professionisti |
| `TC05` | Cassa nazionale del notariato |
| `TC06` | Cassa nazionale previdenza e assistenza ragionieri e periti commerciali |
| `TC07` | Ente nazionale assistenza agenti e rappresentanti di commercio (ENASARCO) |
| `TC08` | Ente nazionale previdenza e assistenza consulenti del lavoro (ENPACL) |
| `TC09` | Ente nazionale previdenza e assistenza medici (ENPAM) |
| `TC10` | Ente nazionale previdenza e assistenza farmacisti (ENPAF) |
| `TC11` | Ente nazionale previdenza e assistenza veterinari (ENPAV) |
| `TC12` | Ente nazionale previdenza e assistenza impiegati dell'agricoltura (ENPAIA) |
| `TC13` | Fondo previdenza impiegati imprese di spedizione e agenzie marittime |
| `TC14` | Istituto nazionale previdenza giornalisti italiani (INPGI) |
| `TC15` | Opera nazionale assistenza orfani sanitari italiani (ONAOSI) |
| `TC16` | Cassa autonoma assistenza integrativa giornalisti italiani (CASAGIT) |
| `TC17` | Ente previdenza periti industriali e periti industriali laureati (EPPI) |
| `TC18` | Ente previdenza e assistenza pluricategoriale (EPAP) |
| `TC19` | Ente nazionale previdenza e assistenza biologi (ENPAB) |
| `TC20` | Ente nazionale previdenza e assistenza professione infermieristica (ENPAPI) |
| `TC21` | Ente nazionale previdenza e assistenza psicologi (ENPAP) |
| `TC22` | INPS |

* * *

## 8. `ModalitaPagamento` (4 caratteri)

L'XSD le elenca in quest'ordine con queste descrizioni [V] — mappatura posizionale `MP01…MP23`:
| Codice | Descrizione |
| --- | --- |
| `MP01` | contanti |
| `MP02` | assegno |
| `MP03` | assegno circolare |
| `MP04` | contanti presso Tesoreria |
| `MP05` | bonifico |
| `MP06` | vaglia cambiario |
| `MP07` | bollettino bancario |
| `MP08` | carta di pagamento |
| `MP09` | RID |
| `MP10` | RID utenze |
| `MP11` | RID veloce |
| `MP12` | RIBA |
| `MP13` | MAV |
| `MP14` | quietanza erario |
| `MP15` | giroconto su conti di contabilità speciale |
| `MP16` | domiciliazione bancaria |
| `MP17` | domiciliazione postale |
| `MP18` | bollettino di c/c postale |
| `MP19` | SEPA Direct Debit |
| `MP20` | SEPA Direct Debit CORE |
| `MP21` | SEPA Direct Debit B2B |
| `MP22` | Trattenuta su somme già riscosse |
| `MP23` | PagoPA |

`MP19–MP23` introdotte dalla **1.6** (04/05/2020). Nota: `MP12` "RIBA" è un "Ricevuta Bancaria", non un RID.

* * *

## 9. Altre enumerazioni chiuse

| Elemento | Valori ammessi | Descrizione ufficiale |
| --- | --- | --- |
| `CondizioniPagamento` | `TP01` · `TP02` · `TP03` | pagamento a rate · pagamento completo · anticipo |
| `EsigibilitaIVA` (solo in `DatiRiepilogo`) | `I` · `D` · `S` | esigibilità immediata · esigibilità differita · **scissione dei pagamenti** |
| `TipoCessionePrestazione` | `SC` · `PR` · `AB` · `AC` | Sconto · Premio · Abbuono · **Spesa accessoria** |
| `TipoScontoMaggiorazione` | `SC` · `MG` | Sconto · Maggiorazione |
| `Art73` | `SI` | _"Documento emesso secondo modalità e termini stabiliti con DM ai sensi dell'art. 73 DPR 633/72"_ |
| `BolloVirtuale` | `SI` | assolvimento dell'imposta di bollo (DM MEF 17/06/2014) |
| `Ritenuta` (in `DettaglioLinee` e `DatiCassaPrevidenziale`) | `SI` | _"Cessione / Prestazione soggetta a ritenuta"_ |
| `SoggettoEmittente` | `CC` · `TZ` | Cessionario/Committente · Terzo |
| `SocioUnico` | `SU` · `SM` | socio unico · **più soci** (⚠️ **non esiste "NO"**) |
| `StatoLiquidazione` | `LS` · `LN` | in liquidazione · non in liquidazione |
| `AlgoritmoCompressione` | stringa libera ≤ 10 | **nessuna enumerazione** |
| `FormatoAttachment` | stringa libera ≤ 10 | **nessuna enumerazione/whitelist nello schema** |

⚠️ Due errori tipici: usare `NO` per `SocioUnico` (il valore è `SM`) e usare `Sì`/`SI` con spazi o minuscole (i valori sono esattamente `SI`, maiuscolo, senza spazi).

* * *

## 10. `AltriDatiGestionali/TipoDato` — "quasi-enumerazione" documentata

Il tipo è `String10Type` (libero per l'XSD) ma la specifica documenta i valori ammessi [S, dal changelog ufficiale e dalle specifiche]:
| `TipoDato` | Significato | Introdotto |
| --- | --- | --- |
| `TARGA` | targa del veicolo (`RiferimentoTesto` = targa) | 1.1 |
| `N.DOC.COMM` | riferimento a documento commerciale/scontrino (`RiferimentoTesto` = id, `RiferimentoData` = data) | 1.6.3 |
| `INVCONT` | art. 21 c.6-bis lett. a) DPR 633/72 | 1.6.3 |
| `INTENTO` | operazione non imponibile ex art. 8 c.1 lett. c) DPR 633/72 (esportatore abituale); `RiferimentoTesto` = protocollo-progressivo della dichiarazione d'intento (→ **00477** se invalidata) | 1.6.3/1.7 |
| `NB1` `NB2` `NB3` | bollo non dovuto / già assorbito (documenti assicurativi; enti del terzo settore; conto corrente bancario-cliente) | 1.6.3 |
| `F24` | debito IVA assolto con F24 | 1.7.1 |
| `NellAnno` `AnniPreced` | estrazione da Deposito IVA (stesso/periodi precedenti) | 1.7.1 |
| `ALI-COMP` | regime speciale agricolo, Tabella A parte prima — **richiede** `RiferimentoNumero` = percentuale di compensazione | 1.8 |
| `NO-COMP` | prodotti agricoli non in Tabella A | 1.8 |
| `OCC34BIS` | operazioni occasionali art. 34-bis | 1.8 |
| `ESENZSPORT` | redditi in esenzione dei lavoratori sportivi (art. 36 c.6 D.Lgs. 36/2021, fino a 15.000 €/anno) | **1.9.1** |

* * *

## 11. `CodiceDestinatario`: spazio di codici e convenzioni

**Formato:** `[A-Z0-9]{6,7}` (ordinaria) / `[A-Z0-9]{7}` (semplificata) → vincolato dalla lunghezza al `FormatoTrasmissione` (**00427**). Case-sensitive: **solo maiuscole**.
| Valore | Significato | Note |
| --- | --- | --- |
| **7 caratteri alfanumerici assegnati dal SdI** | canale accreditato in ricezione, richiesto dal destinatario con la funzione _"Richiesta codici destinatario B2B"_ | dal 1.9.1 un soggetto può richiederne fino a **300** |
| `0000000` | destinatario che riceve **via PEC** e questa è indicata in `PECDestinatario`; **oppure** destinatario di cui **non si conosce il canale** | in quest'ultimo caso il file è reso disponibile nell'area riservata del destinatario |
| `XXXXXXX` | fattura emessa verso **soggetti non residenti / non stabiliti / non identificati in Italia**, inviata al SdI per assolvere la comunicazione transfrontaliera | **richiede** `IdPaese ≠ IT` del cessionario (controllo **00313**) |
| **6 caratteri (codice ufficio IPA)** | PA destinataria: deve corrispondere al _Codice Ufficio_ dell'**IndicePA** (`indicepa.gov.it`) | controlli **00311/00312/00398/00399** — **online** |
| `999999` | convenzione operativa PA: usata quando il codice ufficio non è identificabile | **[S]** documentata in FAQ/supporto, **non** enumerata nell'XSD né nelle specifiche |
| `2R4GTO8` | codice destinatario dedicato San Marino | **[S]** |

**Regola** `PECDestinatario`**:**

*   può essere valorizzata **solo** se `CodiceDestinatario = 0000000` (il che rende **vietata** la PEC quando il codice è un canale accreditato o `XXXXXXX`);
    
*   **non è obbligatoria** quando il codice è `0000000` (il testo 1.9.1 prevede esplicitamente il caso "canale non conosciuto") [V]; il controllo **00426** nella sua versione "PEC obbligatoria con 0000000" era presente nell'Elenco controlli **v1.8** [V] ma risulta **assente** dall'edizione **v2.0** [S] → **trattare solo la direzione "PEC ⇒ 0000000" come vincolo, e verificare**;
    
*   gli indirizzi PEC delle caselle SdI (`sdixx@pec.fatturapa.it`) sono rifiutati (**00330**) — replicabile con una lista locale;
    
*   dal 01/07/2022 il destinatario finale può registrare un "indirizzo telematico" preferito in Fatture e Corrispettivi.
    

* * *

## 12. Codici "a lista aperta" (validabili solo per forma)

| Campo | Regola | Validazione offline |
| --- | --- | --- |
| `Divisa` | `[A-Z]{3}` — ISO 4217 alpha-3 | forma sì; **ammissibilità**: embeddare la lista ISO 4217 |
| `Nazione` / `IdPaese` / `IdPaese` in `IdFiscaleIVA` | `[A-Z]{2}` — ISO 3166-1 alpha-2 | forma sì; **ammissibilità**: embeddare la lista ISO 3166-1 |
| `TipoResa` | `[A-Z]{3}` — codifica Incoterms ICC, **nessuna enumerazione chiusa** | forma sì; ammissibilità = lista Incoterms |
| `Provincia` / `ProvinciaAlbo` / `Ufficio` (REA) | `[A-Z]{2}` | forma sì; **ammissibilità**: embeddare la lista delle sigle |
| `FormatoAttachment` | stringa ≤ 10 | nessuna whitelist nello schema; le implementazioni reali applicano whitelist di estensione/tipo [S] |

**Casi "sensibili" sui codici paese** (documentati nella pratica, da gestire esplicitamente):

*   **Grecia**: ISO alpha-2 = `GR`, ma nelle partite IVA comunitarie si usa `EL` → molte implementazioni accettano **entrambi** [S];
    
*   **Kosovo**: `XK` (codice _user-assigned_, non ISO ufficiale) — presente in diverse liste operative [S];
    
*   `OO`: convenzione per **Livigno e Campione d'Italia** (ammessa in `IdPaese` per TD17/TD19) [S];
    
*   **Irlanda/Islanda**: storici errori di tabella (Islanda mancante, Irlanda duplicata) [S];
    
*   **Irlanda del Nord**: `XI`, **non** `IX` [S];
    
*   numeri di partita IVA che iniziano con `EU`: caso limite ancora oggetto di fix nelle librerie nel 2026 [S].
    
*   L'Agenzia delle Entrate ha **anche** una propria tabella di codici numerici a 3 cifre per i paesi esteri: **non è** quella usata da `Nazione`/`IdPaese` (che è ISO 3166-1 alpha-2). Non confonderle.
    

* * *

## 13. Dove vivono le liste machine-readable e come mantenerle

| Fonte | Cosa contiene | Note di manutenzione |
| --- | --- | --- |
| **XSD ufficiali** (`Schema_VFPR12`, `Schema VFSM10`, `Schema FatturaPA 1.2.3`) | enumerazioni chiuse (TipoDocumento, Natura, RegimeFiscale, TipoRitenuta, TipoCassa, ModalitaPagamento, CondizioniPagamento, EsigibilitaIVA, TipoCessionePrestazione, TipoScontoMaggiorazione, Art73, BolloVirtuale, SocioUnico, StatoLiquidazione, SoggettoEmittente) + tutte le regex/lunghezze | riscaricare a ogni provvedimento; **verificare l'hash del file** |
| **"Rappresentazione tabellare del tracciato" (XLS)** — ordinaria e semplificata, agg. 31/03/2026 | il campo-per-campo con descrizione funzionale e vincoli: **la migliore fonte per l'estrazione automatica dei vincoli** | aggiornata con le specifiche |
| **Allegato A — Specifiche tecniche 1.9.1** (PDF) + Appendici 1-4 | regole di compilazione, controlli, XSD in appendice, notifiche | il documento normativo |
| **"Elenco controlli"** (v1.8 01/10/2022 · v2.0 31/01/2025) | i codici di errore **con pseudocodice in stile COBOL** per i controlli complessi | l'edizione v2.0 **non** contiene ancora 00327/ESENZSPORT di 1.9.1: va integrata con l'Appendice 1 di Allegato A 1.9.1 |
| **"Variazioni alle specifiche tecniche"** (changelog AdE) | l'unica fonte per datare l'introduzione/abrogazione dei codici | da consultare prima di ogni release |
| **Istruzioni modello CU** (annuale) | le descrizioni delle `CausalePagamento` | **cambia ogni anno** |
| **ISO 3166-1 alpha-2 / ISO 4217 / lista sigle province** | tabelle "aperte" | aggiornamento periodico; embeddare la versione |
| **IndicePA (Open Data IPA, CKAN/SPARQL)** | codici ufficio a 6 caratteri per la PA | dataset scaricabile → consente un controllo **offline** di appartenenza (ma non di "attività" nel senso SdI) |
| **Archivio Comuni e Stati esteri AdE** | codici catastali (Belfiore) e codici `Z` degli Stati esteri | serve per validare il codice fiscale delle persone fisiche |

**Ordine di priorità per un'implementazione**: (1) XSD ufficiale → (2) tabella tabellare XLS → (3) Appendice 1 dei controlli → (4) changelog per le date di decorrenza → (5) tabelle esterne (ISO, IPA, Belfiore).