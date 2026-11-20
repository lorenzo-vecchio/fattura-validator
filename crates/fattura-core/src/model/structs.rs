//! Typed nodes of the FPA12/FPR12 invoice tree.
//!
//! Every struct corresponds to one complex type of the official schema
//! (`P1 §6`). The mapping is one-to-one and documented on each type: an element
//! that the XSD declares `minOccurs="0"` is an `Option`, an element with
//! `maxOccurs="unbounded"` is a [`List`], and required elements are `Option`
//! too — the model records what a document *contains*, while requiredness and
//! order are the schema layer's job (E3.4). This keeps a malformed invoice
//! parseable so L3 rules can still run and report every problem at once.
//!
//! Numeric leaves are already parsed: money is [`Amount`], percentages are
//! [`Rate`], `xs:date` fields are [`CivilDate`]. A numeric field is `None` when
//! the element is absent **or** its lexical form does not parse; the schema walk
//! reports the latter with the element path. Dates are parsed the same way.
//! `xs:dateTime` fields (`DataOraRitiro`, `DataOraConsegna`) keep the raw text:
//! their lexical form is checked by E3.3.

use crate::num::{Amount, Rate};

use super::date::CivilDate;
use super::types::{List, Text, UnknownElement};

/// `IdFiscaleType`: a country code plus a fiscal identifier.
///
/// `P1 §6` (`IdTrasmittente`, `IdFiscaleIVA`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IdFiscale<'a> {
    /// `IdPaese` — ISO 3166-1 alpha-2 country code (`1..1`).
    pub id_paese: Option<Text<'a>>,
    /// `IdCodice` — fiscal identifier (`1..1`).
    pub id_codice: Option<Text<'a>>,
}

/// `ContattiType`: telephone, fax and e-mail contacts.
///
/// `P1 §6` (`Contatti` under `CedentePrestatore`; `ContattiTrasmittente`
/// reuses the same shape without the fax field).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Contatti<'a> {
    /// `Telefono` (`0..1`).
    pub telefono: Option<Text<'a>>,
    /// `Fax` (`0..1`).
    pub fax: Option<Text<'a>>,
    /// `Email` (`0..1`).
    pub email: Option<Text<'a>>,
}

/// `DatiTrasmissioneType`: transmission routing data.
///
/// `P1 §6`, order fixed by `P1 §8`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatiTrasmissione<'a> {
    /// `IdTrasmittente` (`1..1`).
    pub id_trasmittente: Option<IdFiscale<'a>>,
    /// `ProgressivoInvio` (`1..1`).
    pub progressivo_invio: Option<Text<'a>>,
    /// `FormatoTrasmissione` (`1..1`): `FPA12`, `FPR12` or `FSM10`.
    pub formato_trasmissione: Option<Text<'a>>,
    /// `CodiceDestinatario` (`1..1`).
    pub codice_destinatario: Option<Text<'a>>,
    /// `ContattiTrasmittente` (`0..1`).
    pub contatti_trasmittente: Option<Contatti<'a>>,
    /// `PECDestinatario` (`0..1`).
    pub pec_destinatario: Option<Text<'a>>,
}

/// `AnagraficaType`: company name or natural-person name.
///
/// The XSD choice is `Denominazione` **xor** (`Nome` + `Cognome`), followed by
/// optional `Titolo` and `CodEORI` (`P1 §6`, `P1 §8`). The choice is represented
/// as mutually exclusive `Option` fields and enforced by E3.4.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Anagrafica<'a> {
    /// `Denominazione` (`0..1`; alternative to `Nome` + `Cognome`).
    pub denominazione: Option<Text<'a>>,
    /// `Nome` (`0..1`; requires `Cognome`).
    pub nome: Option<Text<'a>>,
    /// `Cognome` (`0..1`; requires `Nome`).
    pub cognome: Option<Text<'a>>,
    /// `Titolo` (`0..1`).
    pub titolo: Option<Text<'a>>,
    /// `CodEORI` (`0..1`).
    pub cod_eori: Option<Text<'a>>,
}

/// `DatiAnagraficiType`: taxpayer identity for seller, buyer and intermediaries.
///
/// The same XSD type is used with different requiredness (`IdFiscaleIVA` is
/// required for the seller and optional for the buyer, `P1 §8`); the model uses
/// `Option` throughout and lets E3.4 apply the context.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatiAnagrafici<'a> {
    /// `IdFiscaleIVA` (`1..1` for the seller, `0..1` for the buyer).
    pub id_fiscale_iva: Option<IdFiscale<'a>>,
    /// `CodiceFiscale` (`0..1`).
    pub codice_fiscale: Option<Text<'a>>,
    /// `Anagrafica` (`1..1`).
    pub anagrafica: Option<Anagrafica<'a>>,
    /// `AlboProfessionale` (`0..1`).
    pub albo_professionale: Option<Text<'a>>,
    /// `ProvinciaAlbo` (`0..1`).
    pub provincia_albo: Option<Text<'a>>,
    /// `NumeroIscrizioneAlbo` (`0..1`).
    pub numero_iscrizione_albo: Option<Text<'a>>,
    /// `DataIscrizioneAlbo` (`0..1`, `xs:date`).
    pub data_iscrizione_albo: Option<CivilDate>,
    /// `RegimeFiscale` (`1..1` for the seller).
    pub regime_fiscale: Option<Text<'a>>,
}

/// `IndirizzoType`: postal address.
///
/// The order is fixed by the XSD (`P1 §6`, `P1 §8`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Indirizzo<'a> {
    /// `Indirizzo` (`1..1`).
    pub indirizzo: Option<Text<'a>>,
    /// `NumeroCivico` (`0..1`).
    pub numero_civico: Option<Text<'a>>,
    /// `CAP` (`1..1`).
    pub cap: Option<Text<'a>>,
    /// `Comune` (`1..1`).
    pub comune: Option<Text<'a>>,
    /// `Provincia` (`0..1`).
    pub provincia: Option<Text<'a>>,
    /// `Nazione` (`1..1`, default `IT`).
    pub nazione: Option<Text<'a>>,
}

/// `IscrizioneREAType`: chamber-of-commerce registration.
///
/// `P1 §6`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IscrizioneRea<'a> {
    /// `Ufficio` (`1..1`).
    pub ufficio: Option<Text<'a>>,
    /// `NumeroREA` (`1..1`).
    pub numero_rea: Option<Text<'a>>,
    /// `CapitaleSociale` (`0..1`, `Amount2DecimalType`).
    pub capitale_sociale: Option<Amount>,
    /// `SocioUnico` (`0..1`).
    pub socio_unico: Option<Text<'a>>,
    /// `StatoLiquidazione` (`1..1`).
    pub stato_liquidazione: Option<Text<'a>>,
}

/// `CedentePrestatoreType`: the seller.
///
/// `P1 §6`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CedentePrestatore<'a> {
    /// `DatiAnagrafici` (`1..1`).
    pub dati_anagrafici: Option<DatiAnagrafici<'a>>,
    /// `Sede` (`1..1`).
    pub sede: Option<Indirizzo<'a>>,
    /// `StabileOrganizzazione` (`0..1`).
    pub stabile_organizzazione: Option<Indirizzo<'a>>,
    /// `IscrizioneREA` (`0..1`).
    pub iscrizione_rea: Option<IscrizioneRea<'a>>,
    /// `Contatti` (`0..1`).
    pub contatti: Option<Contatti<'a>>,
    /// `RiferimentoAmministrazione` (`0..1`).
    pub riferimento_amministrazione: Option<Text<'a>>,
}

/// `RappresentanteFiscaleType` in the header.
///
/// `P1 §6`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RappresentanteFiscale<'a> {
    /// `DatiAnagrafici` (`1..1`).
    pub dati_anagrafici: Option<DatiAnagrafici<'a>>,
}

/// `RappresentanteFiscaleCessionarioType`: a buyer's tax representative.
///
/// Unlike the header type, this node carries `IdFiscaleIVA` directly followed by
/// the `Denominazione` / `Nome`+`Cognome` choice (`P1 §6`, `P1 §8`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RappresentanteFiscaleCessionario<'a> {
    /// `IdFiscaleIVA` (`1..1`).
    pub id_fiscale_iva: Option<IdFiscale<'a>>,
    /// `Denominazione` (`0..1`; alternative to `Nome` + `Cognome`).
    pub denominazione: Option<Text<'a>>,
    /// `Nome` (`0..1`; requires `Cognome`).
    pub nome: Option<Text<'a>>,
    /// `Cognome` (`0..1`; requires `Nome`).
    pub cognome: Option<Text<'a>>,
}

/// `CessionarioCommittenteType`: the buyer.
///
/// `P1 §6`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CessionarioCommittente<'a> {
    /// `DatiAnagrafici` (`1..1`; `IdFiscaleIVA` is `0..1` here).
    pub dati_anagrafici: Option<DatiAnagrafici<'a>>,
    /// `Sede` (`1..1`).
    pub sede: Option<Indirizzo<'a>>,
    /// `StabileOrganizzazione` (`0..1`).
    pub stabile_organizzazione: Option<Indirizzo<'a>>,
    /// `RappresentanteFiscale` (`0..1`).
    pub rappresentante_fiscale: Option<RappresentanteFiscaleCessionario<'a>>,
}

/// `TerzoIntermediarioOSoggettoEmittenteType`.
///
/// `P1 §6`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TerzoIntermediario<'a> {
    /// `DatiAnagrafici` (`1..1`).
    pub dati_anagrafici: Option<DatiAnagrafici<'a>>,
}

/// `FatturaElettronicaHeaderType`: everything before the first body.
///
/// `P1 §6`, order fixed by `P1 §8`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Header<'a> {
    /// `DatiTrasmissione` (`1..1`).
    pub dati_trasmissione: Option<DatiTrasmissione<'a>>,
    /// `CedentePrestatore` (`1..1`).
    pub cedente_prestatore: Option<CedentePrestatore<'a>>,
    /// `RappresentanteFiscale` (`0..1`).
    pub rappresentante_fiscale: Option<RappresentanteFiscale<'a>>,
    /// `CessionarioCommittente` (`1..1`).
    pub cessionario_committente: Option<CessionarioCommittente<'a>>,
    /// `TerzoIntermediarioOSoggettoEmittente` (`0..1`).
    pub terzo_intermediario_o_soggetto_emittente: Option<TerzoIntermediario<'a>>,
    /// `SoggettoEmittente` (`0..1`).
    pub soggetto_emittente: Option<Text<'a>>,
}

/// `DatiRitenutaType`: a withholding-tax block.
///
/// `P1 §6` (`DatiRitenuta` under `DatiGeneraliDocumento`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatiRitenuta<'a> {
    /// `TipoRitenuta` (`1..1`).
    pub tipo_ritenuta: Option<Text<'a>>,
    /// `ImportoRitenuta` (`1..1`, `Amount2DecimalType`).
    pub importo_ritenuta: Option<Amount>,
    /// `AliquotaRitenuta` (`1..1`, `RateType`).
    pub aliquota_ritenuta: Option<Rate>,
    /// `CausalePagamento` (`1..1`).
    pub causale_pagamento: Option<Text<'a>>,
}

/// `DatiBolloType`: virtual stamp duty.
///
/// `P1 §6`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatiBollo<'a> {
    /// `BolloVirtuale` (`1..1`; only `SI`).
    pub bollo_virtuale: Option<Text<'a>>,
    /// `ImportoBollo` (`0..1`, `Amount2DecimalType`).
    pub importo_bollo: Option<Amount>,
}

/// `DatiCassaPrevidenzialeType`: a social-security fund contribution.
///
/// `P1 §6`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatiCassaPrevidenziale<'a> {
    /// `TipoCassa` (`1..1`).
    pub tipo_cassa: Option<Text<'a>>,
    /// `AlCassa` (`1..1`, `RateType`).
    pub al_cassa: Option<Rate>,
    /// `ImportoContributoCassa` (`1..1`, `Amount2DecimalType`).
    pub importo_contributo_cassa: Option<Amount>,
    /// `ImponibileCassa` (`0..1`, `Amount2DecimalType`).
    pub imponibile_cassa: Option<Amount>,
    /// `AliquotaIVA` (`1..1`, `RateType`).
    pub aliquota_iva: Option<Rate>,
    /// `Ritenuta` (`0..1`; only `SI`).
    pub ritenuta: Option<Text<'a>>,
    /// `Natura` (`0..1`).
    pub natura: Option<Text<'a>>,
    /// `RiferimentoAmministrazione` (`0..1`).
    pub riferimento_amministrazione: Option<Text<'a>>,
}

/// `ScontoMaggiorazioneType`: a discount or surcharge.
///
/// At least one of `Percentuale` and `Importo` is required by the official
/// controls, not by the XSD's own cardinality (`P1 §6`, `P3 §3.5`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScontoMaggiorazione<'a> {
    /// `Tipo` (`1..1`; `SC` or `MG`).
    pub tipo: Option<Text<'a>>,
    /// `Percentuale` (`0..1`, `RateType`).
    pub percentuale: Option<Rate>,
    /// `Importo` (`0..1`, `Amount8DecimalType`).
    pub importo: Option<Amount>,
}

/// `DatiGeneraliDocumentoType`: the document header.
///
/// The order shown in `P1 §8` is mandatory in the XSD sequence.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatiGeneraliDocumento<'a> {
    /// `TipoDocumento` (`1..1`).
    pub tipo_documento: Option<Text<'a>>,
    /// `Divisa` (`1..1`).
    pub divisa: Option<Text<'a>>,
    /// `Data` (`1..1`, `xs:date`).
    pub data: Option<CivilDate>,
    /// `Numero` (`1..1`).
    pub numero: Option<Text<'a>>,
    /// `DatiRitenuta` (`0..unbounded`).
    pub dati_ritenuta: List<DatiRitenuta<'a>>,
    /// `DatiBollo` (`0..1`).
    pub dati_bollo: Option<DatiBollo<'a>>,
    /// `DatiCassaPrevidenziale` (`0..unbounded`).
    pub dati_cassa_previdenziale: List<DatiCassaPrevidenziale<'a>>,
    /// `ScontoMaggiorazione` (`0..unbounded`).
    pub sconto_maggiorazione: List<ScontoMaggiorazione<'a>>,
    /// `ImportoTotaleDocumento` (`0..1`, `Amount2DecimalType`).
    pub importo_totale_documento: Option<Amount>,
    /// `Arrotondamento` (`0..1`, `Amount2DecimalType`).
    pub arrotondamento: Option<Amount>,
    /// `Causale` (`0..unbounded`).
    pub causale: List<Text<'a>>,
    /// `Art73` (`0..1`).
    pub art73: Option<Text<'a>>,
}

/// `DatiDocumentiCorrelatiType`: a referenced order, contract or invoice.
///
/// The same shape backs `DatiOrdineAcquisto`, `DatiContratto`,
/// `DatiConvenzione`, `DatiRicezione` and `DatiFattureCollegate` (`P1 §6`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatiDocumentiCorrelati<'a> {
    /// `RiferimentoNumeroLinea` (`0..unbounded`, `1..9999`).
    pub riferimento_numero_linea: List<u32>,
    /// `IdDocumento` (`1..1`).
    pub id_documento: Option<Text<'a>>,
    /// `Data` (`0..1`, `xs:date`).
    pub data: Option<CivilDate>,
    /// `NumItem` (`0..1`).
    pub num_item: Option<Text<'a>>,
    /// `CodiceCommessaConvenzione` (`0..1`).
    pub codice_commessa_convenzione: Option<Text<'a>>,
    /// `CodiceCUP` (`0..1`).
    pub codice_cup: Option<Text<'a>>,
    /// `CodiceCIG` (`0..1`).
    pub codice_cig: Option<Text<'a>>,
}

/// `DatiSALType`: a state-of-advancement reference.
///
/// `P1 §6`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatiSal {
    /// `RiferimentoFase` (`1..1`, `1..999`).
    pub riferimento_fase: Option<u32>,
}

/// `DatiDDTType`: a transport-document reference.
///
/// `P1 §6`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatiDdt<'a> {
    /// `NumeroDDT` (`1..1`).
    pub numero_ddt: Option<Text<'a>>,
    /// `DataDDT` (`1..1`, `xs:date`).
    pub data_ddt: Option<CivilDate>,
    /// `RiferimentoNumeroLinea` (`0..unbounded`, `1..9999`).
    pub riferimento_numero_linea: List<u32>,
}

/// `DatiAnagraficiVettoreType`: the carrier's identity.
///
/// `P1 §6` (`DatiAnagraficiVettore` under `DatiTrasporto`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatiAnagraficiVettore<'a> {
    /// `IdFiscaleIVA` (`1..1`).
    pub id_fiscale_iva: Option<IdFiscale<'a>>,
    /// `CodiceFiscale` (`0..1`).
    pub codice_fiscale: Option<Text<'a>>,
    /// `Anagrafica` (`1..1`).
    pub anagrafica: Option<Anagrafica<'a>>,
    /// `NumeroLicenzaGuida` (`0..1`).
    pub numero_licenza_guida: Option<Text<'a>>,
}

/// `DatiTrasportoType`: transport data.
///
/// All fields are optional in the XSD (`P1 §6`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatiTrasporto<'a> {
    /// `DatiAnagraficiVettore` (`0..1`).
    pub dati_anagrafici_vettore: Option<DatiAnagraficiVettore<'a>>,
    /// `MezzoTrasporto` (`0..1`).
    pub mezzo_trasporto: Option<Text<'a>>,
    /// `CausaleTrasporto` (`0..1`).
    pub causale_trasporto: Option<Text<'a>>,
    /// `NumeroColli` (`0..1`, `1..9999`).
    pub numero_colli: Option<u32>,
    /// `Descrizione` (`0..1`).
    pub descrizione: Option<Text<'a>>,
    /// `UnitaMisuraPeso` (`0..1`).
    pub unita_misura_peso: Option<Text<'a>>,
    /// `PesoLordo` (`0..1`, `PesoType`).
    pub peso_lordo: Option<Amount>,
    /// `PesoNetto` (`0..1`, `PesoType`).
    pub peso_netto: Option<Amount>,
    /// `DataOraRitiro` (`0..1`, `xs:dateTime`; lexical form checked by E3.3).
    pub data_ora_ritiro: Option<Text<'a>>,
    /// `DataInizioTrasporto` (`0..1`, `xs:date`).
    pub data_inizio_trasporto: Option<CivilDate>,
    /// `TipoResa` (`0..1`, three uppercase letters).
    pub tipo_resa: Option<Text<'a>>,
    /// `IndirizzoResa` (`0..1`).
    pub indirizzo_resa: Option<Indirizzo<'a>>,
    /// `DataOraConsegna` (`0..1`, `xs:dateTime`; lexical form checked by E3.3).
    pub data_ora_consegna: Option<Text<'a>>,
}

/// `FatturaPrincipaleType`: the main invoice for a down-payment document.
///
/// `P1 §6`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FatturaPrincipale<'a> {
    /// `NumeroFatturaPrincipale` (`1..1`).
    pub numero_fattura_principale: Option<Text<'a>>,
    /// `DataFatturaPrincipale` (`1..1`, `xs:date`).
    pub data_fattura_principale: Option<CivilDate>,
}

/// `DatiGeneraliType`: document-level data and references.
///
/// The order of the five `DatiDocumentiCorrelati` groups is fixed by the XSD
/// (`P1 §8`); each group keeps its original order in its own [`List`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatiGenerali<'a> {
    /// `DatiGeneraliDocumento` (`1..1`).
    pub dati_generali_documento: Option<DatiGeneraliDocumento<'a>>,
    /// `DatiOrdineAcquisto` (`0..unbounded`).
    pub dati_ordine_acquisto: List<DatiDocumentiCorrelati<'a>>,
    /// `DatiContratto` (`0..unbounded`).
    pub dati_contratto: List<DatiDocumentiCorrelati<'a>>,
    /// `DatiConvenzione` (`0..unbounded`).
    pub dati_convenzione: List<DatiDocumentiCorrelati<'a>>,
    /// `DatiRicezione` (`0..unbounded`).
    pub dati_ricezione: List<DatiDocumentiCorrelati<'a>>,
    /// `DatiFattureCollegate` (`0..unbounded`).
    pub dati_fatture_collegate: List<DatiDocumentiCorrelati<'a>>,
    /// `DatiSAL` (`0..unbounded`).
    pub dati_sal: List<DatiSal>,
    /// `DatiDDT` (`0..unbounded`).
    pub dati_ddt: List<DatiDdt<'a>>,
    /// `DatiTrasporto` (`0..1`).
    pub dati_trasporto: Option<DatiTrasporto<'a>>,
    /// `FatturaPrincipale` (`0..1`).
    pub fattura_principale: Option<FatturaPrincipale<'a>>,
}

/// `CodiceArticoloType`: an article code and its code-list name.
///
/// `P1 §6` (`CodiceArticolo` under `DettaglioLinee`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CodiceArticolo<'a> {
    /// `CodiceTipo` (`1..1`).
    pub codice_tipo: Option<Text<'a>>,
    /// `CodiceValore` (`1..1`; `String35LatinExtType`, the one extended string).
    pub codice_valore: Option<Text<'a>>,
}

/// `AltriDatiGestionaliType`: free-form line metadata.
///
/// `P1 §6`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AltriDatiGestionali<'a> {
    /// `TipoDato` (`1..1`).
    pub tipo_dato: Option<Text<'a>>,
    /// `RiferimentoTesto` (`0..1`).
    pub riferimento_testo: Option<Text<'a>>,
    /// `RiferimentoNumero` (`0..1`, `Amount8DecimalType`).
    pub riferimento_numero: Option<Amount>,
    /// `RiferimentoData` (`0..1`, `xs:date`).
    pub riferimento_data: Option<CivilDate>,
}

/// `DettaglioLineeType`: one invoice line.
///
/// The field order is the mandatory XSD sequence (`P1 §8`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DettaglioLinee<'a> {
    /// `NumeroLinea` (`1..1`, `1..9999`).
    pub numero_linea: Option<u32>,
    /// `TipoCessionePrestazione` (`0..1`).
    pub tipo_cessione_prestazione: Option<Text<'a>>,
    /// `CodiceArticolo` (`0..unbounded`).
    pub codice_articolo: List<CodiceArticolo<'a>>,
    /// `Descrizione` (`1..1`).
    pub descrizione: Option<Text<'a>>,
    /// `Quantita` (`0..1`, `QuantitaType`).
    pub quantita: Option<Amount>,
    /// `UnitaMisura` (`0..1`).
    pub unita_misura: Option<Text<'a>>,
    /// `DataInizioPeriodo` (`0..1`, `xs:date`).
    pub data_inizio_periodo: Option<CivilDate>,
    /// `DataFinePeriodo` (`0..1`, `xs:date`).
    pub data_fine_periodo: Option<CivilDate>,
    /// `PrezzoUnitario` (`1..1`, `Amount8DecimalType`).
    pub prezzo_unitario: Option<Amount>,
    /// `ScontoMaggiorazione` (`0..unbounded`).
    pub sconto_maggiorazione: List<ScontoMaggiorazione<'a>>,
    /// `PrezzoTotale` (`1..1`, `Amount8DecimalType`).
    pub prezzo_totale: Option<Amount>,
    /// `AliquotaIVA` (`1..1`, `RateType`).
    pub aliquota_iva: Option<Rate>,
    /// `Ritenuta` (`0..1`; only `SI`).
    pub ritenuta: Option<Text<'a>>,
    /// `Natura` (`0..1`).
    pub natura: Option<Text<'a>>,
    /// `RiferimentoAmministrazione` (`0..1`).
    pub riferimento_amministrazione: Option<Text<'a>>,
    /// `AltriDatiGestionali` (`0..unbounded`).
    pub altri_dati_gestionali: List<AltriDatiGestionali<'a>>,
}

/// `DatiRiepilogoType`: VAT summary row.
///
/// `P1 §6`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatiRiepilogo<'a> {
    /// `AliquotaIVA` (`1..1`, `RateType`).
    pub aliquota_iva: Option<Rate>,
    /// `Natura` (`0..1`).
    pub natura: Option<Text<'a>>,
    /// `SpeseAccessorie` (`0..1`, `Amount2DecimalType`).
    pub spese_accessorie: Option<Amount>,
    /// `Arrotondamento` (`0..1`, `Amount8DecimalType`).
    pub arrotondamento: Option<Amount>,
    /// `ImponibileImporto` (`1..1`, `Amount2DecimalType`).
    pub imponibile_importo: Option<Amount>,
    /// `Imposta` (`1..1`, `Amount2DecimalType`).
    pub imposta: Option<Amount>,
    /// `EsigibilitaIVA` (`0..1`; `D`, `I` or `S`).
    pub esigibilita_iva: Option<Text<'a>>,
    /// `RiferimentoNormativo` (`0..1`).
    pub riferimento_normativo: Option<Text<'a>>,
}

/// `DatiBeniServiziType`: lines and VAT summary.
///
/// `P1 §6`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatiBeniServizi<'a> {
    /// `DettaglioLinee` (`1..unbounded`).
    pub dettaglio_linee: List<DettaglioLinee<'a>>,
    /// `DatiRiepilogo` (`1..unbounded`).
    pub dati_riepilogo: List<DatiRiepilogo<'a>>,
}

/// `DatiVeicoliType`: vehicle data for the `TD22` document type.
///
/// `P1 §6`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatiVeicoli<'a> {
    /// `Data` (`1..1`, `xs:date`).
    pub data: Option<CivilDate>,
    /// `TotalePercorso` (`1..1`).
    pub totale_percorso: Option<Text<'a>>,
}

/// `DettaglioPagamentoType`: one payment instalment.
///
/// The order is the XSD sequence listed in `P1 §6`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DettaglioPagamento<'a> {
    /// `Beneficiario` (`0..1`).
    pub beneficiario: Option<Text<'a>>,
    /// `ModalitaPagamento` (`1..1`).
    pub modalita_pagamento: Option<Text<'a>>,
    /// `DataRiferimentoTerminiPagamento` (`0..1`, `xs:date`).
    pub data_riferimento_termini_pagamento: Option<CivilDate>,
    /// `GiorniTerminiPagamento` (`0..1`, `0..999`).
    pub giorni_termini_pagamento: Option<u32>,
    /// `DataScadenzaPagamento` (`0..1`, `xs:date`).
    pub data_scadenza_pagamento: Option<CivilDate>,
    /// `ImportoPagamento` (`1..1`, `Amount2DecimalType`).
    pub importo_pagamento: Option<Amount>,
    /// `CodUfficioPostale` (`0..1`).
    pub cod_ufficio_postale: Option<Text<'a>>,
    /// `CognomeQuietanzante` (`0..1`).
    pub cognome_quietanzante: Option<Text<'a>>,
    /// `NomeQuietanzante` (`0..1`).
    pub nome_quietanzante: Option<Text<'a>>,
    /// `CFQuietanzante` (`0..1`; exactly 16 characters when present).
    pub cf_quietanzante: Option<Text<'a>>,
    /// `TitoloQuietanzante` (`0..1`).
    pub titolo_quietanzante: Option<Text<'a>>,
    /// `IstitutoFinanziario` (`0..1`).
    pub istituto_finanziario: Option<Text<'a>>,
    /// `IBAN` (`0..1`).
    pub iban: Option<Text<'a>>,
    /// `ABI` (`0..1`, five digits).
    pub abi: Option<Text<'a>>,
    /// `CAB` (`0..1`, five digits).
    pub cab: Option<Text<'a>>,
    /// `BIC` (`0..1`).
    pub bic: Option<Text<'a>>,
    /// `ScontoPagamentoAnticipato` (`0..1`, `Amount2DecimalType`).
    pub sconto_pagamento_anticipato: Option<Amount>,
    /// `DataLimitePagamentoAnticipato` (`0..1`, `xs:date`).
    pub data_limite_pagamento_anticipato: Option<CivilDate>,
    /// `PenalitaPagamentiRitardati` (`0..1`, `Amount2DecimalType`).
    pub penalita_pagamenti_ritardati: Option<Amount>,
    /// `DataDecorrenzaPenale` (`0..1`, `xs:date`).
    pub data_decorrenza_penale: Option<CivilDate>,
    /// `CodicePagamento` (`0..1`).
    pub codice_pagamento: Option<Text<'a>>,
}

/// `DatiPagamentoType`: one payment-terms block.
///
/// `P1 §6`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatiPagamento<'a> {
    /// `CondizioniPagamento` (`1..1`).
    pub condizioni_pagamento: Option<Text<'a>>,
    /// `DettaglioPagamento` (`1..unbounded`).
    pub dettaglio_pagamento: List<DettaglioPagamento<'a>>,
}

/// `AllegatiType`: one attachment.
///
/// `Attachment` is `xs:base64Binary`; the model keeps its textual form and does
/// not decode the payload (`P1 §6`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Allegati<'a> {
    /// `NomeAttachment` (`1..1`).
    pub nome_attachment: Option<Text<'a>>,
    /// `AlgoritmoCompressione` (`0..1`).
    pub algoritmo_compressione: Option<Text<'a>>,
    /// `FormatoAttachment` (`0..1`).
    pub formato_attachment: Option<Text<'a>>,
    /// `DescrizioneAttachment` (`0..1`).
    pub descrizione_attachment: Option<Text<'a>>,
    /// `Attachment` (`1..1`, `xs:base64Binary`, raw text).
    pub attachment: Option<Text<'a>>,
}

/// `FatturaElettronicaBodyType`: one invoice body.
///
/// A lotto file carries one or more of these; the position in
/// [`Document::bodies`](super::Document::bodies) is the 1-based body index used
/// by finding paths (`P1 §6`, `P1 §8`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Body<'a> {
    /// `DatiGenerali` (`1..1`).
    pub dati_generali: Option<DatiGenerali<'a>>,
    /// `DatiBeniServizi` (`1..1`).
    pub dati_beni_servizi: Option<DatiBeniServizi<'a>>,
    /// `DatiVeicoli` (`0..1`).
    pub dati_veicoli: Option<DatiVeicoli<'a>>,
    /// `DatiPagamento` (`0..unbounded`).
    pub dati_pagamento: List<DatiPagamento<'a>>,
    /// `Allegati` (`0..unbounded`).
    pub allegati: List<Allegati<'a>>,
}

/// `FatturaElettronica`: the document root and its attributes.
///
/// `@versione` and `@SistemaEmittente` are the only attributes the model keeps
/// (`P1 §6`). The root's namespace declarations are not part of the model;
/// namespace resolution is reporting-only (`P1 §5.1`).
///
/// The `ds:Signature` subtree is not modelled: only its presence is recorded,
/// because signature verification belongs to E8 (`P1 §10`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Document<'a> {
    /// `@versione` (`1..1`): `FPA12`, `FPR12` or `FSM10`.
    pub versione: Option<Text<'a>>,
    /// `@SistemaEmittente` (`0..1`).
    pub sistema_emittente: Option<Text<'a>>,
    /// `FatturaElettronicaHeader` (`1..1`).
    pub header: Option<Header<'a>>,
    /// `FatturaElettronicaBody` (`1..unbounded`); order preserved.
    pub bodies: List<Body<'a>>,
    /// `ds:Signature` is present (`0..1`).
    pub signature: bool,
    /// Elements that are not part of the FPA12/FPR12 tree, in document order.
    pub unknown: List<UnknownElement<'a>>,
}
