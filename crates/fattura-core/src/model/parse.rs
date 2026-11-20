//! Streaming builder for the typed model.
//!
//! The parser reuses the E2.1 [`Reader`] and consumes events in one pass. It
//! never builds a DOM: each `parse_*` function reads exactly its element's
//! children and returns a typed struct. Element order is preserved because
//! repeated children are appended in document order; the 1-based index used by
//! findings is therefore the element's position in the [`List`](super::List).
//!
//! Matching is on local names, namespace-agnostic, matching the reader
//! (`P1 §5.1`). Elements the FPA12/FPR12 tree does not define at that position
//! are recorded in [`Document::unknown`](super::Document::unknown) and skipped;
//! they are not interpreted.
//!
//! Text leaves are borrowed (`Cow::Borrowed`) unless they arrive in several runs
//! or an entity reference forces a decoded copy. Numeric and date leaves are
//! converted here; a lexical failure yields `None` and is reported by the schema
//! layer.

use std::borrow::Cow;

use crate::num::{Amount, Rate};
use crate::xml::{Element, EntityRef, Event, Limits, Reader};

use super::date::CivilDate;
use super::error::ModelError;
use super::structs::{
    Allegati, AltriDatiGestionali, Anagrafica, Body, CedentePrestatore, CessionarioCommittente,
    CodiceArticolo, Contatti, DatiAnagrafici, DatiAnagraficiVettore, DatiBeniServizi, DatiBollo,
    DatiCassaPrevidenziale, DatiDdt, DatiDocumentiCorrelati, DatiGenerali, DatiGeneraliDocumento,
    DatiPagamento, DatiRiepilogo, DatiRitenuta, DatiSal, DatiTrasmissione, DatiTrasporto,
    DatiVeicoli, DettaglioLinee, DettaglioPagamento, Document, FatturaPrincipale, Header,
    IdFiscale, Indirizzo, IscrizioneRea, RappresentanteFiscale, RappresentanteFiscaleCessionario,
    ScontoMaggiorazione, TerzoIntermediario,
};
use super::types::{List, Text, UnknownElement};

/// Builds the typed model from a complete invoice document.
///
/// The parser reads until the root element closes. The returned [`Document`]
/// borrows from `input`; only decoded entity references and the input-independent
/// parts of the report allocate.
///
/// # Errors
///
/// Returns [`ModelError`] when the XML is not well-formed, the root is not
/// `FatturaElettronica`, the input ends inside an element, or a text leaf is not
/// valid UTF-8. Lexically invalid numeric or date values do not fail the parse;
/// they become `None` so the schema layer can report every occurrence.
pub fn parse(input: &[u8]) -> Result<Document<'_>, ModelError> {
    let reader = Reader::new(input, Limits::default());
    let mut parser = Parser::new(reader);
    parser.parse_document()
}

/// One reader event with self-closing tags normalised to `Start` + `End`.
///
/// `quick-xml` distinguishes `<x/>` from `<x></x>`; the model does not. A
/// self-closing tag is reported as `Start` and makes the next call return
/// `End`, so every `parse_*` function has a single shape and a single set of
/// branches.
enum Node<'a> {
    Start(Element<'a>),
    End,
    Text(crate::xml::Text<'a>),
    Entity(EntityRef<'a>),
    /// End of input or a reader failure. The first error, if any, is stored in
    /// [`Parser::error`].
    Abort,
}

struct Parser<'a> {
    reader: Reader<'a>,
    synthetic_end: bool,
    unknown: List<UnknownElement<'a>>,
    error: Option<ModelError>,
}

impl<'a> Parser<'a> {
    fn new(reader: Reader<'a>) -> Self {
        Self {
            reader,
            synthetic_end: false,
            unknown: List::new(),
            error: None,
        }
    }

    /// Keeps the first fatal error; later failures cannot replace the cause.
    fn record_error(&mut self, error: ModelError) {
        if self.error.is_none() {
            self.error = Some(error);
        }
    }

    fn next_node(&mut self) -> Node<'a> {
        if self.synthetic_end {
            self.synthetic_end = false;
            return Node::End;
        }
        let event = match self.reader.read_event() {
            Ok(event) => event,
            Err(error) => {
                self.record_error(ModelError::Xml(error));
                return Node::Abort;
            }
        };
        match event {
            Event::Start(element) => Node::Start(element),
            Event::Empty(element) => {
                self.synthetic_end = true;
                Node::Start(element)
            }
            Event::End(_) => Node::End,
            Event::Text(text) => Node::Text(text),
            Event::EntityRef(entity) => Node::Entity(entity),
            Event::Eof => Node::Abort,
        }
    }

    fn parse_document(&mut self) -> Result<Document<'a>, ModelError> {
        let root = loop {
            match self.next_node() {
                Node::Start(element) => break element,
                Node::Abort => {
                    if let Some(error) = self.error.take() {
                        return Err(error);
                    }
                    return Err(ModelError::MissingRoot);
                }
                Node::End | Node::Text(_) | Node::Entity(_) => {}
            }
        };

        if root.name().local() != b"FatturaElettronica" {
            return Err(ModelError::UnsupportedRoot);
        }

        let mut document = Document::default();
        read_root_attributes(&root, &mut document);
        self.children(|parser, child| parser.parse_root_child(&mut document, &child));
        document.unknown = core::mem::take(&mut self.unknown);
        if let Some(error) = self.error.take() {
            return Err(error);
        }
        Ok(document)
    }

    fn parse_root_child(&mut self, document: &mut Document<'a>, element: &Element<'a>) {
        match element.name().local() {
            b"FatturaElettronicaHeader" => {
                document.header = Some(self.parse_header());
            }
            b"FatturaElettronicaBody" => {
                document.bodies.push(self.parse_body());
            }
            b"Signature" => {
                // The XAdES subtree is E8's concern; only its presence has to
                // survive into the model.
                document.signature = true;
                self.skip_children();
            }
            _ => self.skip_unknown(element),
        }
    }

    /// Consumes children of the current element, calling `visit` for each.
    fn children<F>(&mut self, mut visit: F)
    where
        F: FnMut(&mut Self, Element<'a>),
    {
        loop {
            match self.next_node() {
                Node::Start(element) => visit(self, element),
                Node::End | Node::Abort => return,
                Node::Text(_) | Node::Entity(_) => {}
            }
        }
    }

    fn skip_children(&mut self) {
        loop {
            match self.next_node() {
                Node::Start(_) => self.skip_children(),
                Node::End | Node::Abort => return,
                Node::Text(_) | Node::Entity(_) => {}
            }
        }
    }

    fn skip_unknown(&mut self, element: &Element<'a>) {
        self.unknown.push(UnknownElement::new(
            element.name(),
            element.offset(),
            element.depth(),
        ));
        self.skip_children();
    }

    fn take_text_event(&mut self, value: &mut Option<Text<'a>>, text: crate::xml::Text<'a>) {
        if let Err(error) = take_text(value, text) {
            self.record_error(error);
        }
    }

    fn append_entity_event(&mut self, value: &mut Option<Text<'a>>, entity: &EntityRef<'a>) {
        if let Err(error) = append_entity(value, entity) {
            self.record_error(error);
        }
    }

    fn text(&mut self) -> Text<'a> {
        let mut value: Option<Text<'a>> = None;
        loop {
            match self.next_node() {
                Node::Text(text) => self.take_text_event(&mut value, text),
                Node::Entity(entity) => self.append_entity_event(&mut value, &entity),
                Node::End | Node::Abort => return value.unwrap_or(Cow::Borrowed("")),
                Node::Start(_) => {
                    self.record_error(ModelError::UnexpectedElement);
                    return value.unwrap_or(Cow::Borrowed(""));
                }
            }
        }
    }

    fn amount8(&mut self) -> Option<Amount> {
        let text = self.text();
        Amount::parse_amount8(text.as_ref()).ok()
    }

    fn amount2(&mut self) -> Option<Amount> {
        let text = self.text();
        Amount::parse_amount2(text.as_ref()).ok()
    }

    fn rate(&mut self) -> Option<Rate> {
        let text = self.text();
        Rate::parse(text.as_ref()).ok()
    }

    fn date(&mut self) -> Option<CivilDate> {
        let text = self.text();
        CivilDate::parse(text.as_ref()).ok()
    }

    fn integer(&mut self) -> Option<u32> {
        let text = self.text();
        text.parse::<u32>().ok()
    }

    fn parse_header(&mut self) -> Header<'a> {
        let mut out = Header::default();
        self.children(|parser, child| match child.name().local() {
            b"DatiTrasmissione" => {
                out.dati_trasmissione = Some(parser.parse_dati_trasmissione());
            }
            b"CedentePrestatore" => {
                out.cedente_prestatore = Some(parser.parse_cedente_prestatore());
            }
            b"RappresentanteFiscale" => {
                out.rappresentante_fiscale = Some(parser.parse_rappresentante_fiscale());
            }
            b"CessionarioCommittente" => {
                out.cessionario_committente = Some(parser.parse_cessionario_committente());
            }
            b"TerzoIntermediarioOSoggettoEmittente" => {
                out.terzo_intermediario_o_soggetto_emittente =
                    Some(parser.parse_terzo_intermediario());
            }
            b"SoggettoEmittente" => {
                out.soggetto_emittente = Some(parser.text());
            }
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_dati_trasmissione(&mut self) -> DatiTrasmissione<'a> {
        let mut out = DatiTrasmissione::default();
        self.children(|parser, child| match child.name().local() {
            b"IdTrasmittente" => {
                out.id_trasmittente = Some(parser.parse_id_fiscale());
            }
            b"ProgressivoInvio" => out.progressivo_invio = Some(parser.text()),
            b"FormatoTrasmissione" => out.formato_trasmissione = Some(parser.text()),
            b"CodiceDestinatario" => out.codice_destinatario = Some(parser.text()),
            b"ContattiTrasmittente" => {
                out.contatti_trasmittente = Some(parser.parse_contatti());
            }
            b"PECDestinatario" => out.pec_destinatario = Some(parser.text()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_id_fiscale(&mut self) -> IdFiscale<'a> {
        let mut out = IdFiscale::default();
        self.children(|parser, child| match child.name().local() {
            b"IdPaese" => out.id_paese = Some(parser.text()),
            b"IdCodice" => out.id_codice = Some(parser.text()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_contatti(&mut self) -> Contatti<'a> {
        let mut out = Contatti::default();
        self.children(|parser, child| match child.name().local() {
            b"Telefono" => out.telefono = Some(parser.text()),
            b"Fax" => out.fax = Some(parser.text()),
            b"Email" => out.email = Some(parser.text()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_anagrafica(&mut self) -> Anagrafica<'a> {
        let mut out = Anagrafica::default();
        self.children(|parser, child| match child.name().local() {
            b"Denominazione" => out.denominazione = Some(parser.text()),
            b"Nome" => out.nome = Some(parser.text()),
            b"Cognome" => out.cognome = Some(parser.text()),
            b"Titolo" => out.titolo = Some(parser.text()),
            b"CodEORI" => out.cod_eori = Some(parser.text()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_dati_anagrafici(&mut self) -> DatiAnagrafici<'a> {
        let mut out = DatiAnagrafici::default();
        self.children(|parser, child| match child.name().local() {
            b"IdFiscaleIVA" => out.id_fiscale_iva = Some(parser.parse_id_fiscale()),
            b"CodiceFiscale" => out.codice_fiscale = Some(parser.text()),
            b"Anagrafica" => out.anagrafica = Some(parser.parse_anagrafica()),
            b"AlboProfessionale" => out.albo_professionale = Some(parser.text()),
            b"ProvinciaAlbo" => out.provincia_albo = Some(parser.text()),
            b"NumeroIscrizioneAlbo" => out.numero_iscrizione_albo = Some(parser.text()),
            b"DataIscrizioneAlbo" => out.data_iscrizione_albo = parser.date(),
            b"RegimeFiscale" => out.regime_fiscale = Some(parser.text()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_indirizzo(&mut self) -> Indirizzo<'a> {
        let mut out = Indirizzo::default();
        self.children(|parser, child| match child.name().local() {
            b"Indirizzo" => out.indirizzo = Some(parser.text()),
            b"NumeroCivico" => out.numero_civico = Some(parser.text()),
            b"CAP" => out.cap = Some(parser.text()),
            b"Comune" => out.comune = Some(parser.text()),
            b"Provincia" => out.provincia = Some(parser.text()),
            b"Nazione" => out.nazione = Some(parser.text()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_iscrizione_rea(&mut self) -> IscrizioneRea<'a> {
        let mut out = IscrizioneRea::default();
        self.children(|parser, child| match child.name().local() {
            b"Ufficio" => out.ufficio = Some(parser.text()),
            b"NumeroREA" => out.numero_rea = Some(parser.text()),
            b"CapitaleSociale" => out.capitale_sociale = parser.amount2(),
            b"SocioUnico" => out.socio_unico = Some(parser.text()),
            b"StatoLiquidazione" => out.stato_liquidazione = Some(parser.text()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_cedente_prestatore(&mut self) -> CedentePrestatore<'a> {
        let mut out = CedentePrestatore::default();
        self.children(|parser, child| match child.name().local() {
            b"DatiAnagrafici" => {
                out.dati_anagrafici = Some(parser.parse_dati_anagrafici());
            }
            b"Sede" => out.sede = Some(parser.parse_indirizzo()),
            b"StabileOrganizzazione" => {
                out.stabile_organizzazione = Some(parser.parse_indirizzo());
            }
            b"IscrizioneREA" => {
                out.iscrizione_rea = Some(parser.parse_iscrizione_rea());
            }
            b"Contatti" => out.contatti = Some(parser.parse_contatti()),
            b"RiferimentoAmministrazione" => {
                out.riferimento_amministrazione = Some(parser.text());
            }
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_rappresentante_fiscale(&mut self) -> RappresentanteFiscale<'a> {
        let mut out = RappresentanteFiscale::default();
        self.children(|parser, child| match child.name().local() {
            b"DatiAnagrafici" => {
                out.dati_anagrafici = Some(parser.parse_dati_anagrafici());
            }
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_rappresentante_fiscale_cessionario(&mut self) -> RappresentanteFiscaleCessionario<'a> {
        let mut out = RappresentanteFiscaleCessionario::default();
        self.children(|parser, child| match child.name().local() {
            b"IdFiscaleIVA" => out.id_fiscale_iva = Some(parser.parse_id_fiscale()),
            b"Denominazione" => out.denominazione = Some(parser.text()),
            b"Nome" => out.nome = Some(parser.text()),
            b"Cognome" => out.cognome = Some(parser.text()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_cessionario_committente(&mut self) -> CessionarioCommittente<'a> {
        let mut out = CessionarioCommittente::default();
        self.children(|parser, child| match child.name().local() {
            b"DatiAnagrafici" => {
                out.dati_anagrafici = Some(parser.parse_dati_anagrafici());
            }
            b"Sede" => out.sede = Some(parser.parse_indirizzo()),
            b"StabileOrganizzazione" => {
                out.stabile_organizzazione = Some(parser.parse_indirizzo());
            }
            b"RappresentanteFiscale" => {
                out.rappresentante_fiscale =
                    Some(parser.parse_rappresentante_fiscale_cessionario());
            }
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_terzo_intermediario(&mut self) -> TerzoIntermediario<'a> {
        let mut out = TerzoIntermediario::default();
        self.children(|parser, child| match child.name().local() {
            b"DatiAnagrafici" => {
                out.dati_anagrafici = Some(parser.parse_dati_anagrafici());
            }
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_body(&mut self) -> Body<'a> {
        let mut out = Body::default();
        self.children(|parser, child| match child.name().local() {
            b"DatiGenerali" => out.dati_generali = Some(parser.parse_dati_generali()),
            b"DatiBeniServizi" => {
                out.dati_beni_servizi = Some(parser.parse_dati_beni_servizi());
            }
            b"DatiVeicoli" => out.dati_veicoli = Some(parser.parse_dati_veicoli()),
            b"DatiPagamento" => {
                out.dati_pagamento.push(parser.parse_dati_pagamento());
            }
            b"Allegati" => out.allegati.push(parser.parse_allegati()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_dati_generali(&mut self) -> DatiGenerali<'a> {
        let mut out = DatiGenerali::default();
        self.children(|parser, child| match child.name().local() {
            b"DatiGeneraliDocumento" => {
                out.dati_generali_documento = Some(parser.parse_dati_generali_documento());
            }
            b"DatiOrdineAcquisto" => out
                .dati_ordine_acquisto
                .push(parser.parse_dati_documenti_correlati()),
            b"DatiContratto" => out
                .dati_contratto
                .push(parser.parse_dati_documenti_correlati()),
            b"DatiConvenzione" => out
                .dati_convenzione
                .push(parser.parse_dati_documenti_correlati()),
            b"DatiRicezione" => out
                .dati_ricezione
                .push(parser.parse_dati_documenti_correlati()),
            b"DatiFattureCollegate" => out
                .dati_fatture_collegate
                .push(parser.parse_dati_documenti_correlati()),
            b"DatiSAL" => out.dati_sal.push(parser.parse_dati_sal()),
            b"DatiDDT" => out.dati_ddt.push(parser.parse_dati_ddt()),
            b"DatiTrasporto" => {
                out.dati_trasporto = Some(parser.parse_dati_trasporto());
            }
            b"FatturaPrincipale" => {
                out.fattura_principale = Some(parser.parse_fattura_principale());
            }
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_dati_generali_documento(&mut self) -> DatiGeneraliDocumento<'a> {
        let mut out = DatiGeneraliDocumento::default();
        self.children(|parser, child| match child.name().local() {
            b"TipoDocumento" => out.tipo_documento = Some(parser.text()),
            b"Divisa" => out.divisa = Some(parser.text()),
            b"Data" => out.data = parser.date(),
            b"Numero" => out.numero = Some(parser.text()),
            b"DatiRitenuta" => {
                out.dati_ritenuta.push(parser.parse_dati_ritenuta());
            }
            b"DatiBollo" => out.dati_bollo = Some(parser.parse_dati_bollo()),
            b"DatiCassaPrevidenziale" => out
                .dati_cassa_previdenziale
                .push(parser.parse_dati_cassa_previdenziale()),
            b"ScontoMaggiorazione" => out
                .sconto_maggiorazione
                .push(parser.parse_sconto_maggiorazione()),
            b"ImportoTotaleDocumento" => out.importo_totale_documento = parser.amount2(),
            b"Arrotondamento" => out.arrotondamento = parser.amount2(),
            b"Causale" => out.causale.push(parser.text()),
            b"Art73" => out.art73 = Some(parser.text()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_dati_ritenuta(&mut self) -> DatiRitenuta<'a> {
        let mut out = DatiRitenuta::default();
        self.children(|parser, child| match child.name().local() {
            b"TipoRitenuta" => out.tipo_ritenuta = Some(parser.text()),
            b"ImportoRitenuta" => out.importo_ritenuta = parser.amount2(),
            b"AliquotaRitenuta" => out.aliquota_ritenuta = parser.rate(),
            b"CausalePagamento" => out.causale_pagamento = Some(parser.text()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_dati_bollo(&mut self) -> DatiBollo<'a> {
        let mut out = DatiBollo::default();
        self.children(|parser, child| match child.name().local() {
            b"BolloVirtuale" => out.bollo_virtuale = Some(parser.text()),
            b"ImportoBollo" => out.importo_bollo = parser.amount2(),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_dati_cassa_previdenziale(&mut self) -> DatiCassaPrevidenziale<'a> {
        let mut out = DatiCassaPrevidenziale::default();
        self.children(|parser, child| match child.name().local() {
            b"TipoCassa" => out.tipo_cassa = Some(parser.text()),
            b"AlCassa" => out.al_cassa = parser.rate(),
            b"ImportoContributoCassa" => out.importo_contributo_cassa = parser.amount2(),
            b"ImponibileCassa" => out.imponibile_cassa = parser.amount2(),
            b"AliquotaIVA" => out.aliquota_iva = parser.rate(),
            b"Ritenuta" => out.ritenuta = Some(parser.text()),
            b"Natura" => out.natura = Some(parser.text()),
            b"RiferimentoAmministrazione" => {
                out.riferimento_amministrazione = Some(parser.text());
            }
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_sconto_maggiorazione(&mut self) -> ScontoMaggiorazione<'a> {
        let mut out = ScontoMaggiorazione::default();
        self.children(|parser, child| match child.name().local() {
            b"Tipo" => out.tipo = Some(parser.text()),
            b"Percentuale" => out.percentuale = parser.rate(),
            b"Importo" => out.importo = parser.amount8(),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_dati_documenti_correlati(&mut self) -> DatiDocumentiCorrelati<'a> {
        let mut out = DatiDocumentiCorrelati::default();
        self.children(|parser, child| match child.name().local() {
            b"RiferimentoNumeroLinea" => {
                if let Some(line) = parser.integer() {
                    out.riferimento_numero_linea.push(line);
                }
            }
            b"IdDocumento" => out.id_documento = Some(parser.text()),
            b"Data" => out.data = parser.date(),
            b"NumItem" => out.num_item = Some(parser.text()),
            b"CodiceCommessaConvenzione" => {
                out.codice_commessa_convenzione = Some(parser.text());
            }
            b"CodiceCUP" => out.codice_cup = Some(parser.text()),
            b"CodiceCIG" => out.codice_cig = Some(parser.text()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_dati_sal(&mut self) -> DatiSal {
        let mut out = DatiSal::default();
        self.children(|parser, child| match child.name().local() {
            b"RiferimentoFase" => {
                out.riferimento_fase = parser.integer();
            }
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_dati_ddt(&mut self) -> DatiDdt<'a> {
        let mut out = DatiDdt::default();
        self.children(|parser, child| match child.name().local() {
            b"NumeroDDT" => out.numero_ddt = Some(parser.text()),
            b"DataDDT" => out.data_ddt = parser.date(),
            b"RiferimentoNumeroLinea" => {
                if let Some(line) = parser.integer() {
                    out.riferimento_numero_linea.push(line);
                }
            }
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_dati_anagrafici_vettore(&mut self) -> DatiAnagraficiVettore<'a> {
        let mut out = DatiAnagraficiVettore::default();
        self.children(|parser, child| match child.name().local() {
            b"IdFiscaleIVA" => out.id_fiscale_iva = Some(parser.parse_id_fiscale()),
            b"CodiceFiscale" => out.codice_fiscale = Some(parser.text()),
            b"Anagrafica" => out.anagrafica = Some(parser.parse_anagrafica()),
            b"NumeroLicenzaGuida" => out.numero_licenza_guida = Some(parser.text()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_dati_trasporto(&mut self) -> DatiTrasporto<'a> {
        let mut out = DatiTrasporto::default();
        self.children(|parser, child| match child.name().local() {
            b"DatiAnagraficiVettore" => {
                out.dati_anagrafici_vettore = Some(parser.parse_dati_anagrafici_vettore());
            }
            b"MezzoTrasporto" => out.mezzo_trasporto = Some(parser.text()),
            b"CausaleTrasporto" => out.causale_trasporto = Some(parser.text()),
            b"NumeroColli" => out.numero_colli = parser.integer(),
            b"Descrizione" => out.descrizione = Some(parser.text()),
            b"UnitaMisuraPeso" => out.unita_misura_peso = Some(parser.text()),
            b"PesoLordo" => out.peso_lordo = parser.amount2(),
            b"PesoNetto" => out.peso_netto = parser.amount2(),
            b"DataOraRitiro" => out.data_ora_ritiro = Some(parser.text()),
            b"DataInizioTrasporto" => out.data_inizio_trasporto = parser.date(),
            b"TipoResa" => out.tipo_resa = Some(parser.text()),
            b"IndirizzoResa" => out.indirizzo_resa = Some(parser.parse_indirizzo()),
            b"DataOraConsegna" => out.data_ora_consegna = Some(parser.text()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_fattura_principale(&mut self) -> FatturaPrincipale<'a> {
        let mut out = FatturaPrincipale::default();
        self.children(|parser, child| match child.name().local() {
            b"NumeroFatturaPrincipale" => {
                out.numero_fattura_principale = Some(parser.text());
            }
            b"DataFatturaPrincipale" => out.data_fattura_principale = parser.date(),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_dati_beni_servizi(&mut self) -> DatiBeniServizi<'a> {
        let mut out = DatiBeniServizi::default();
        self.children(|parser, child| match child.name().local() {
            b"DettaglioLinee" => {
                out.dettaglio_linee.push(parser.parse_dettaglio_linee());
            }
            b"DatiRiepilogo" => {
                out.dati_riepilogo.push(parser.parse_dati_riepilogo());
            }
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_codice_articolo(&mut self) -> CodiceArticolo<'a> {
        let mut out = CodiceArticolo::default();
        self.children(|parser, child| match child.name().local() {
            b"CodiceTipo" => out.codice_tipo = Some(parser.text()),
            b"CodiceValore" => out.codice_valore = Some(parser.text()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_altri_dati_gestionali(&mut self) -> AltriDatiGestionali<'a> {
        let mut out = AltriDatiGestionali::default();
        self.children(|parser, child| match child.name().local() {
            b"TipoDato" => out.tipo_dato = Some(parser.text()),
            b"RiferimentoTesto" => out.riferimento_testo = Some(parser.text()),
            b"RiferimentoNumero" => out.riferimento_numero = parser.amount8(),
            b"RiferimentoData" => out.riferimento_data = parser.date(),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_dettaglio_linee(&mut self) -> DettaglioLinee<'a> {
        let mut out = DettaglioLinee::default();
        self.children(|parser, child| match child.name().local() {
            b"NumeroLinea" => out.numero_linea = parser.integer(),
            b"TipoCessionePrestazione" => {
                out.tipo_cessione_prestazione = Some(parser.text());
            }
            b"CodiceArticolo" => {
                out.codice_articolo.push(parser.parse_codice_articolo());
            }
            b"Descrizione" => out.descrizione = Some(parser.text()),
            b"Quantita" => out.quantita = parser.amount8(),
            b"UnitaMisura" => out.unita_misura = Some(parser.text()),
            b"DataInizioPeriodo" => out.data_inizio_periodo = parser.date(),
            b"DataFinePeriodo" => out.data_fine_periodo = parser.date(),
            b"PrezzoUnitario" => out.prezzo_unitario = parser.amount8(),
            b"ScontoMaggiorazione" => out
                .sconto_maggiorazione
                .push(parser.parse_sconto_maggiorazione()),
            b"PrezzoTotale" => out.prezzo_totale = parser.amount8(),
            b"AliquotaIVA" => out.aliquota_iva = parser.rate(),
            b"Ritenuta" => out.ritenuta = Some(parser.text()),
            b"Natura" => out.natura = Some(parser.text()),
            b"RiferimentoAmministrazione" => {
                out.riferimento_amministrazione = Some(parser.text());
            }
            b"AltriDatiGestionali" => out
                .altri_dati_gestionali
                .push(parser.parse_altri_dati_gestionali()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_dati_riepilogo(&mut self) -> DatiRiepilogo<'a> {
        let mut out = DatiRiepilogo::default();
        self.children(|parser, child| match child.name().local() {
            b"AliquotaIVA" => out.aliquota_iva = parser.rate(),
            b"Natura" => out.natura = Some(parser.text()),
            b"SpeseAccessorie" => out.spese_accessorie = parser.amount2(),
            b"Arrotondamento" => out.arrotondamento = parser.amount8(),
            b"ImponibileImporto" => out.imponibile_importo = parser.amount2(),
            b"Imposta" => out.imposta = parser.amount2(),
            b"EsigibilitaIVA" => out.esigibilita_iva = Some(parser.text()),
            b"RiferimentoNormativo" => out.riferimento_normativo = Some(parser.text()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_dati_veicoli(&mut self) -> DatiVeicoli<'a> {
        let mut out = DatiVeicoli::default();
        self.children(|parser, child| match child.name().local() {
            b"Data" => out.data = parser.date(),
            b"TotalePercorso" => out.totale_percorso = Some(parser.text()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_dati_pagamento(&mut self) -> DatiPagamento<'a> {
        let mut out = DatiPagamento::default();
        self.children(|parser, child| match child.name().local() {
            b"CondizioniPagamento" => out.condizioni_pagamento = Some(parser.text()),
            b"DettaglioPagamento" => {
                out.dettaglio_pagamento
                    .push(parser.parse_dettaglio_pagamento());
            }
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_dettaglio_pagamento(&mut self) -> DettaglioPagamento<'a> {
        let mut out = DettaglioPagamento::default();
        self.children(|parser, child| match child.name().local() {
            b"Beneficiario" => out.beneficiario = Some(parser.text()),
            b"ModalitaPagamento" => out.modalita_pagamento = Some(parser.text()),
            b"DataRiferimentoTerminiPagamento" => {
                out.data_riferimento_termini_pagamento = parser.date();
            }
            b"GiorniTerminiPagamento" => out.giorni_termini_pagamento = parser.integer(),
            b"DataScadenzaPagamento" => out.data_scadenza_pagamento = parser.date(),
            b"ImportoPagamento" => out.importo_pagamento = parser.amount2(),
            b"CodUfficioPostale" => out.cod_ufficio_postale = Some(parser.text()),
            b"CognomeQuietanzante" => out.cognome_quietanzante = Some(parser.text()),
            b"NomeQuietanzante" => out.nome_quietanzante = Some(parser.text()),
            b"CFQuietanzante" => out.cf_quietanzante = Some(parser.text()),
            b"TitoloQuietanzante" => out.titolo_quietanzante = Some(parser.text()),
            b"IstitutoFinanziario" => out.istituto_finanziario = Some(parser.text()),
            b"IBAN" => out.iban = Some(parser.text()),
            b"ABI" => out.abi = Some(parser.text()),
            b"CAB" => out.cab = Some(parser.text()),
            b"BIC" => out.bic = Some(parser.text()),
            b"ScontoPagamentoAnticipato" => {
                out.sconto_pagamento_anticipato = parser.amount2();
            }
            b"DataLimitePagamentoAnticipato" => {
                out.data_limite_pagamento_anticipato = parser.date();
            }
            b"PenalitaPagamentiRitardati" => {
                out.penalita_pagamenti_ritardati = parser.amount2();
            }
            b"DataDecorrenzaPenale" => out.data_decorrenza_penale = parser.date(),
            b"CodicePagamento" => out.codice_pagamento = Some(parser.text()),
            _ => parser.skip_unknown(&child),
        });
        out
    }

    fn parse_allegati(&mut self) -> Allegati<'a> {
        let mut out = Allegati::default();
        self.children(|parser, child| match child.name().local() {
            b"NomeAttachment" => out.nome_attachment = Some(parser.text()),
            b"AlgoritmoCompressione" => {
                out.algoritmo_compressione = Some(parser.text());
            }
            b"FormatoAttachment" => out.formato_attachment = Some(parser.text()),
            b"DescrizioneAttachment" => {
                out.descrizione_attachment = Some(parser.text());
            }
            b"Attachment" => out.attachment = Some(parser.text()),
            _ => parser.skip_unknown(&child),
        });
        out
    }
}

/// Reads `@versione` and `@SistemaEmittente`, ignoring namespace declarations.
///
/// Attribute values borrow from the parser's event buffer rather than from the
/// input slice, so the two root values are copied once. They are the only
/// attribute values the model keeps.
fn read_root_attributes(element: &Element<'_>, document: &mut Document<'_>) {
    for attribute in element.attributes().filter_map(Result::ok) {
        let name = attribute.name();
        if name.prefix().is_some() {
            continue;
        }
        let value = core::str::from_utf8(attribute.value())
            .ok()
            .map(|text| Cow::Owned(text.to_owned()));
        match name.local() {
            b"versione" => document.versione = value,
            b"SistemaEmittente" => document.sistema_emittente = value,
            _ => {}
        }
    }
}

/// Appends one reader text node, borrowing when the reader kept the bytes.
fn take_text<'a>(
    value: &mut Option<Text<'a>>,
    text: crate::xml::Text<'a>,
) -> Result<(), ModelError> {
    match text.into_raw() {
        Cow::Borrowed(bytes) => append_text(value, bytes),
        Cow::Owned(bytes) => append_owned(value, &bytes),
    }
}

fn append_text<'a>(value: &mut Option<Text<'a>>, bytes: &'a [u8]) -> Result<(), ModelError> {
    let chunk = core::str::from_utf8(bytes).map_err(|_| ModelError::InvalidUtf8)?;
    match value {
        None => *value = Some(Cow::Borrowed(chunk)),
        Some(Cow::Borrowed(previous)) => {
            let mut owned = String::with_capacity(previous.len().saturating_add(chunk.len()));
            owned.push_str(previous);
            owned.push_str(chunk);
            *value = Some(Cow::Owned(owned));
        }
        Some(Cow::Owned(owned)) => owned.push_str(chunk),
    }
    Ok(())
}

/// Appends a text node the reader had to own; the bytes are copied into the
/// model's owned string because the reader buffer does not outlive the event.
fn append_owned(value: &mut Option<Text<'_>>, bytes: &[u8]) -> Result<(), ModelError> {
    let chunk = core::str::from_utf8(bytes).map_err(|_| ModelError::InvalidUtf8)?;
    match value {
        None => *value = Some(Cow::Owned(chunk.to_owned())),
        Some(Cow::Borrowed(previous)) => {
            let mut owned = String::with_capacity(previous.len().saturating_add(chunk.len()));
            owned.push_str(previous);
            owned.push_str(chunk);
            *value = Some(Cow::Owned(owned));
        }
        Some(Cow::Owned(owned)) => owned.push_str(chunk),
    }
    Ok(())
}

fn append_entity<'a>(
    value: &mut Option<Text<'a>>,
    entity: &EntityRef<'a>,
) -> Result<(), ModelError> {
    let character = match entity.resolve_char_ref() {
        Ok(Some(character)) => character,
        // Named references resolve to `None`; only the five XML predefined
        // entities exist because DTDs are rejected before this point.
        Ok(None) => match entity.raw() {
            b"amp" => '&',
            b"lt" => '<',
            b"gt" => '>',
            b"apos" => '\'',
            b"quot" => '"',
            _ => return Err(ModelError::UndefinedEntity),
        },
        Err(_) => return Err(ModelError::UndefinedEntity),
    };

    match value {
        None => {
            let mut owned = String::new();
            owned.push(character);
            *value = Some(Cow::Owned(owned));
        }
        Some(Cow::Borrowed(previous)) => {
            let mut owned = String::with_capacity(previous.len().saturating_add(1));
            owned.push_str(previous);
            owned.push(character);
            *value = Some(Cow::Owned(owned));
        }
        Some(Cow::Owned(owned)) => owned.push(character),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::assert_is_empty
    )]

    use std::borrow::Cow;

    use crate::xml::Text as XmlText;

    use super::{Text, take_text};

    fn owned_text(bytes: &[u8]) -> XmlText<'static> {
        XmlText::new(Cow::Owned(bytes.to_vec()), false, 1, 0)
    }

    #[test]
    fn copied_reader_text_fills_every_owned_state() {
        let mut value: Option<Text<'static>> = None;
        take_text(&mut value, owned_text(b"abc")).unwrap();
        assert_eq!(value.as_deref(), Some("abc"));

        let mut value: Option<Text<'static>> = Some(Cow::Borrowed("x"));
        take_text(&mut value, owned_text(b"y")).unwrap();
        assert_eq!(value.as_deref(), Some("xy"));

        take_text(&mut value, owned_text(b"z")).unwrap();
        assert_eq!(value.as_deref(), Some("xyz"));
    }

    #[test]
    fn borrowed_reader_text_is_rejected_when_not_utf8() {
        let mut value: Option<Text<'static>> = None;
        let text = XmlText::new(Cow::Borrowed(&[0xff][..]), false, 1, 0);
        assert_eq!(
            take_text(&mut value, text).unwrap_err().to_string(),
            "text is not valid UTF-8"
        );
    }

    #[test]
    fn owned_reader_text_is_rejected_when_not_utf8() {
        let mut value: Option<Text<'static>> = None;
        assert_eq!(
            take_text(&mut value, owned_text(&[0xff]))
                .unwrap_err()
                .to_string(),
            "text is not valid UTF-8"
        );
    }
}
