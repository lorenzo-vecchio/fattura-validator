//! Typed document model for FPA12/FPR12 and FSM10.
//!
//! The model keeps borrowed slices from the input where possible and uses
//! `SmallVec` for per-body collections. It represents the multi-body *lotto*
//! natively: one `FatturaElettronicaBody` is not a special case.
//!
//! # Entry point
//!
//! [`parse`] turns an invoice's bytes into a [`Document`] in one streaming pass.
//! It reads through the E2.1 [`Reader`](crate::xml::Reader) and never builds a
//! DOM. The returned model borrows from the input; only entity-decoded or split
//! text, the two root attributes, and collections that outgrow their inline
//! slot allocate.
//!
//! ```
//! use fattura_core::model::parse;
//!
//! let xml = br#"<FatturaElettronica versione="FPR12"><FatturaElettronicaHeader/><FatturaElettronicaBody/></FatturaElettronica>"#;
//! let document = parse(xml).expect("well-formed input");
//! assert_eq!(document.versione.as_deref(), Some("FPR12"));
//! assert_eq!(document.bodies.len(), 1);
//! ```
//!
//! # Node → struct → cardinality
//!
//! The tree is a direct transcription of `P1 §6`; order and requiredness live in
//! `P1 §8` and are enforced by the schema layer (E3.4). Every field is `Option`
//! so an invalid document still produces a model; a [`List`] preserves document
//! order and therefore the 1-based index used in finding paths.
//!
//! | XML parent | Struct | Cardinality |
//! | --- | --- | --- |
//! | `FatturaElettronica` | [`Document`] | root |
//! | `FatturaElettronicaHeader` | [`Header`] | 1..1 |
//! | `DatiTrasmissione` | [`DatiTrasmissione`] | 1..1 |
//! | `IdTrasmittente`, `IdFiscaleIVA` | [`IdFiscale`] | 1..1 / 0..1 by context |
//! | `Contatti`, `ContattiTrasmittente` | [`Contatti`] | 0..1 |
//! | `CedentePrestatore` | [`CedentePrestatore`] | 1..1 |
//! | `RappresentanteFiscale` | [`RappresentanteFiscale`] | 0..1 |
//! | `CessionarioCommittente` | [`CessionarioCommittente`] | 1..1 |
//! | `RappresentanteFiscale` (buyer) | [`RappresentanteFiscaleCessionario`] | 0..1 |
//! | `TerzoIntermediarioOSoggettoEmittente` | [`TerzoIntermediario`] | 0..1 |
//! | `DatiAnagrafici` | [`DatiAnagrafici`] | 1..1 by context |
//! | `Anagrafica` | [`Anagrafica`] | 1..1 |
//! | `Sede`, `StabileOrganizzazione`, `IndirizzoResa` | [`Indirizzo`] | 1..1 / 0..1 |
//! | `IscrizioneREA` | [`IscrizioneRea`] | 0..1 |
//! | `FatturaElettronicaBody` | [`Body`] | 1..unbounded |
//! | `DatiGenerali` | [`DatiGenerali`] | 1..1 |
//! | `DatiGeneraliDocumento` | [`DatiGeneraliDocumento`] | 1..1 |
//! | `DatiRitenuta` | [`DatiRitenuta`] | 0..unbounded |
//! | `DatiBollo` | [`DatiBollo`] | 0..1 |
//! | `DatiCassaPrevidenziale` | [`DatiCassaPrevidenziale`] | 0..unbounded |
//! | `ScontoMaggiorazione` | [`ScontoMaggiorazione`] | 0..unbounded |
//! | `DatiOrdineAcquisto`, `DatiContratto`, `DatiConvenzione`, `DatiRicezione`, `DatiFattureCollegate` | [`DatiDocumentiCorrelati`] | 0..unbounded |
//! | `DatiSAL` | [`DatiSal`] | 0..unbounded |
//! | `DatiDDT` | [`DatiDdt`] | 0..unbounded |
//! | `DatiTrasporto` | [`DatiTrasporto`] | 0..1 |
//! | `DatiAnagraficiVettore` | [`DatiAnagraficiVettore`] | 0..1 |
//! | `FatturaPrincipale` | [`FatturaPrincipale`] | 0..1 |
//! | `DatiBeniServizi` | [`DatiBeniServizi`] | 1..1 |
//! | `DettaglioLinee` | [`DettaglioLinee`] | 1..unbounded |
//! | `CodiceArticolo` | [`CodiceArticolo`] | 0..unbounded |
//! | `AltriDatiGestionali` | [`AltriDatiGestionali`] | 0..unbounded |
//! | `DatiRiepilogo` | [`DatiRiepilogo`] | 1..unbounded |
//! | `DatiVeicoli` | [`DatiVeicoli`] | 0..1 |
//! | `DatiPagamento` | [`DatiPagamento`] | 0..unbounded |
//! | `DettaglioPagamento` | [`DettaglioPagamento`] | 1..unbounded |
//! | `Allegati` | [`Allegati`] | 0..unbounded |
//!
//! # Numeric and date semantics
//!
//! * Money → [`Amount`](crate::num::Amount) (exact scale-8 `i128`).
//! * Percentages and rates → [`Rate`](crate::num::Rate) (exact `0.00..=100.00`).
//! * `xs:date` → [`CivilDate`] (pure calendar arithmetic, no clock, no floating point).
//! * `xs:dateTime` → raw text; its lexical form belongs to E3.3.
//!
//! A numeric or date field is `None` when the element is absent *or* when its
//! lexical form does not parse. The schema walk reports the latter with the
//! element path, so L3 rules can keep operating on the rest of the document.
//!
//! # Unknown elements
//!
//! Elements outside the FPA12/FPR12 tree for their position are recorded in
//! [`Document::unknown`] with their byte offset and depth, and skipped. This is
//! the evidence the XSD layer needs; the model never guesses a meaning.
//!
//! # Stability
//!
//! Internal crate API until E9.1.

mod date;
mod error;
mod parse;
mod structs;
mod types;

pub use date::{CivilDate, DateError};
pub use error::ModelError;
pub use parse::parse;
pub use structs::{
    Allegati, AltriDatiGestionali, Anagrafica, Body, CedentePrestatore, CessionarioCommittente,
    CodiceArticolo, Contatti, DatiAnagrafici, DatiAnagraficiVettore, DatiBeniServizi, DatiBollo,
    DatiCassaPrevidenziale, DatiDdt, DatiDocumentiCorrelati, DatiGenerali, DatiGeneraliDocumento,
    DatiPagamento, DatiRiepilogo, DatiRitenuta, DatiSal, DatiTrasmissione, DatiTrasporto,
    DatiVeicoli, DettaglioLinee, DettaglioPagamento, Document, FatturaPrincipale, Header,
    IdFiscale, Indirizzo, IscrizioneRea, RappresentanteFiscale, RappresentanteFiscaleCessionario,
    ScontoMaggiorazione, TerzoIntermediario,
};
pub use types::{List, Text, UnknownElement};
