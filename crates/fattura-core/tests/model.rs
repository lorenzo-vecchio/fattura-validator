//! Integration tests for the typed FPA12/FPR12 model (E2.2).
//!
//! Fixtures live in `tests/data/model/`; every assertion names the XML node it
//! covers so a failure points at the source tree in `P1 §6`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::assert_is_empty
)]

use fattura_core::model::{CivilDate, Document, ModelError, parse};
use fattura_core::num::{Amount, Rate};

fn amount(text: &str) -> Amount {
    Amount::parse_amount8(text).unwrap()
}

fn amount2(text: &str) -> Amount {
    Amount::parse_amount2(text).unwrap()
}

fn rate(text: &str) -> Rate {
    Rate::parse(text).unwrap()
}

fn complete_document() -> Document<'static> {
    parse(include_bytes!("data/model/complete.xml")).unwrap()
}

#[test]
fn parses_the_minimal_fixture() {
    let document = parse(include_bytes!("data/model/minimal.xml")).unwrap();
    assert_eq!(document.versione.as_deref(), Some("FPR12"));
    assert_eq!(document.sistema_emittente.as_deref(), Some("TEST"));
    assert!(!document.signature);
    assert!(document.unknown.is_empty());

    let header = document.header.as_ref().unwrap();
    let transmission = header.dati_trasmissione.as_ref().unwrap();
    assert_eq!(
        transmission
            .id_trasmittente
            .as_ref()
            .unwrap()
            .id_paese
            .as_deref(),
        Some("IT")
    );
    assert_eq!(transmission.progressivo_invio.as_deref(), Some("00001"));
    assert_eq!(transmission.formato_trasmissione.as_deref(), Some("FPR12"));
    assert_eq!(transmission.codice_destinatario.as_deref(), Some("0000000"));
    assert!(transmission.contatti_trasmittente.is_none());
    assert!(transmission.pec_destinatario.is_none());

    let seller = header.cedente_prestatore.as_ref().unwrap();
    assert_eq!(
        seller
            .dati_anagrafici
            .as_ref()
            .unwrap()
            .regime_fiscale
            .as_deref(),
        Some("RF01")
    );
    assert_eq!(seller.sede.as_ref().unwrap().cap.as_deref(), Some("00100"));

    assert_eq!(document.bodies.len(), 1);
    let body = &document.bodies[0];
    let document_data = body
        .dati_generali
        .as_ref()
        .unwrap()
        .dati_generali_documento
        .as_ref()
        .unwrap();
    assert_eq!(document_data.tipo_documento.as_deref(), Some("TD01"));
    assert_eq!(
        document_data.data,
        Some(CivilDate::from_ymd(2026, 11, 18).unwrap())
    );
    assert_eq!(document_data.numero.as_deref(), Some("1/2026"));

    let lines = &body.dati_beni_servizi.as_ref().unwrap().dettaglio_linee;
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].numero_linea, Some(1));
    assert_eq!(lines[0].prezzo_unitario, Some(amount2("100.00")));
    assert_eq!(lines[0].aliquota_iva, Some(rate("22.00")));
}

#[test]
fn complete_header() {
    let document = complete_document();
    assert_eq!(document.versione.as_deref(), Some("FPA12"));
    assert_eq!(document.sistema_emittente.as_deref(), Some("TEST"));
    assert!(document.signature);

    let header = document.header.as_ref().unwrap();
    let transmission = header.dati_trasmissione.as_ref().unwrap();
    let contacts = transmission.contatti_trasmittente.as_ref().unwrap();
    assert_eq!(contacts.telefono.as_deref(), Some("0612345678"));
    assert_eq!(contacts.email.as_deref(), Some("trasmittente@example.it"));
    assert_eq!(
        transmission.pec_destinatario.as_deref(),
        Some("destinatario@example.it")
    );

    let seller = header.cedente_prestatore.as_ref().unwrap();
    let seller_id = seller.dati_anagrafici.as_ref().unwrap();
    assert_eq!(seller_id.codice_fiscale.as_deref(), Some("01234567890"));
    let anagrafica = seller_id.anagrafica.as_ref().unwrap();
    assert_eq!(
        anagrafica.denominazione.as_deref(),
        Some("Fornitore di prova S.r.l.")
    );
    assert_eq!(anagrafica.titolo.as_deref(), Some("Sig."));
    assert_eq!(anagrafica.cod_eori.as_deref(), Some("IT0123456789012"));
    assert_eq!(
        seller_id.albo_professionale.as_deref(),
        Some("Albo dei consulenti")
    );
    assert_eq!(seller_id.provincia_albo.as_deref(), Some("RM"));
    assert_eq!(seller_id.numero_iscrizione_albo.as_deref(), Some("12345"));
    assert_eq!(
        seller_id.data_iscrizione_albo,
        Some(CivilDate::from_ymd(2024, 2, 29).unwrap())
    );
    assert_eq!(
        seller.sede.as_ref().unwrap().numero_civico.as_deref(),
        Some("10/A")
    );
    assert!(seller.stabile_organizzazione.is_some());
    let rea = seller.iscrizione_rea.as_ref().unwrap();
    assert_eq!(rea.ufficio.as_deref(), Some("RM"));
    assert_eq!(rea.numero_rea.as_deref(), Some("123456"));
    assert_eq!(rea.capitale_sociale, Some(amount2("10000.00")));
    assert_eq!(rea.socio_unico.as_deref(), Some("SU"));
    assert_eq!(rea.stato_liquidazione.as_deref(), Some("LN"));
    let seller_contacts = seller.contatti.as_ref().unwrap();
    assert_eq!(seller_contacts.fax.as_deref(), Some("0612345679"));
    assert_eq!(seller.riferimento_amministrazione.as_deref(), Some("UFF-1"));

    let tax_representative = header.rappresentante_fiscale.as_ref().unwrap();
    assert_eq!(
        tax_representative
            .dati_anagrafici
            .as_ref()
            .unwrap()
            .codice_fiscale
            .as_deref(),
        Some("10987654321")
    );

    let buyer = header.cessionario_committente.as_ref().unwrap();
    let buyer_name = buyer
        .dati_anagrafici
        .as_ref()
        .unwrap()
        .anagrafica
        .as_ref()
        .unwrap();
    assert_eq!(buyer_name.nome.as_deref(), Some("Mario"));
    assert_eq!(buyer_name.cognome.as_deref(), Some("Rossi"));
    assert!(buyer.stabile_organizzazione.is_some());
    let buyer_representative = buyer.rappresentante_fiscale.as_ref().unwrap();
    assert_eq!(
        buyer_representative.denominazione.as_deref(),
        Some("Rappr. Cliente S.r.l.")
    );
    assert_eq!(buyer_representative.nome.as_deref(), Some("Luigi"));
    assert_eq!(buyer_representative.cognome.as_deref(), Some("Bianchi"));

    assert!(header.terzo_intermediario_o_soggetto_emittente.is_some());
    assert_eq!(header.soggetto_emittente.as_deref(), Some("TZ"));
}

#[test]
fn complete_document_data() {
    let document = complete_document();
    assert_eq!(document.bodies.len(), 1);
    let body = &document.bodies[0];
    let general = body.dati_generali.as_ref().unwrap();
    let document_data = general.dati_generali_documento.as_ref().unwrap();
    assert_eq!(
        document_data.data,
        Some(CivilDate::from_ymd(2024, 2, 29).unwrap())
    );
    assert_eq!(document_data.numero.as_deref(), Some("1/2024"));

    let withholding = &document_data.dati_ritenuta;
    assert_eq!(withholding.len(), 1);
    assert_eq!(withholding[0].tipo_ritenuta.as_deref(), Some("RT01"));
    assert_eq!(withholding[0].importo_ritenuta, Some(amount2("20.00")));
    assert_eq!(withholding[0].aliquota_ritenuta, Some(rate("20.00")));
    assert_eq!(withholding[0].causale_pagamento.as_deref(), Some("A"));

    let stamp = document_data.dati_bollo.as_ref().unwrap();
    assert_eq!(stamp.bollo_virtuale.as_deref(), Some("SI"));
    assert_eq!(stamp.importo_bollo, Some(amount2("2.00")));

    let fund = &document_data.dati_cassa_previdenziale;
    assert_eq!(fund.len(), 1);
    assert_eq!(fund[0].tipo_cassa.as_deref(), Some("TC01"));
    assert_eq!(fund[0].al_cassa, Some(rate("4.00")));
    assert_eq!(fund[0].importo_contributo_cassa, Some(amount2("4.00")));
    assert_eq!(fund[0].imponibile_cassa, Some(amount2("100.00")));
    assert_eq!(fund[0].ritenuta.as_deref(), Some("SI"));
    assert_eq!(fund[0].natura.as_deref(), Some("N1"));
    assert_eq!(
        fund[0].riferimento_amministrazione.as_deref(),
        Some("UFF-2")
    );

    let discount = &document_data.sconto_maggiorazione;
    assert_eq!(discount.len(), 1);
    assert_eq!(discount[0].tipo.as_deref(), Some("SC"));
    assert_eq!(discount[0].percentuale, Some(rate("10.00")));
    assert_eq!(discount[0].importo, Some(amount("1.23456789")));

    assert_eq!(
        document_data.importo_totale_documento,
        Some(amount2("122.00"))
    );
    assert_eq!(document_data.arrotondamento, Some(amount2("-0.01")));
    assert_eq!(document_data.causale.len(), 2);
    assert_eq!(document_data.causale[0], "Prima causale");
    assert_eq!(document_data.causale[1], "Seconda & causale");
    assert_eq!(document_data.art73.as_deref(), Some("SI"));

    assert_eq!(general.dati_ordine_acquisto.len(), 1);
    let order = &general.dati_ordine_acquisto[0];
    assert_eq!(order.riferimento_numero_linea.as_slice(), &[1, 2]);
    assert_eq!(order.id_documento.as_deref(), Some("ORD-1"));
    assert_eq!(order.data, Some(CivilDate::from_ymd(2024, 1, 10).unwrap()));
    assert_eq!(order.num_item.as_deref(), Some("1"));
    assert_eq!(order.codice_commessa_convenzione.as_deref(), Some("COMM-1"));
    assert_eq!(order.codice_cup.as_deref(), Some("CUP123"));
    assert_eq!(order.codice_cig.as_deref(), Some("CIG123"));
}

#[test]
fn complete_references_and_transport() {
    let document = complete_document();
    let body = &document.bodies[0];
    let general = body.dati_generali.as_ref().unwrap();
    assert_eq!(
        general.dati_contratto[0].id_documento.as_deref(),
        Some("CON-1")
    );
    assert_eq!(
        general.dati_convenzione[0].id_documento.as_deref(),
        Some("CNV-1")
    );
    assert_eq!(
        general.dati_ricezione[0].id_documento.as_deref(),
        Some("RIC-1")
    );
    assert_eq!(
        general.dati_fatture_collegate[0].id_documento.as_deref(),
        Some("FAT-1")
    );

    assert_eq!(general.dati_sal[0].riferimento_fase, Some(2));
    let ddt = &general.dati_ddt[0];
    assert_eq!(ddt.numero_ddt.as_deref(), Some("DDT-1"));
    assert_eq!(ddt.data_ddt, Some(CivilDate::from_ymd(2024, 2, 1).unwrap()));
    assert_eq!(ddt.riferimento_numero_linea.as_slice(), &[1]);

    let transport = general.dati_trasporto.as_ref().unwrap();
    let carrier = transport.dati_anagrafici_vettore.as_ref().unwrap();
    assert_eq!(carrier.numero_licenza_guida.as_deref(), Some("LIC-1"));
    assert_eq!(transport.mezzo_trasporto.as_deref(), Some("Furgone"));
    assert_eq!(transport.causale_trasporto.as_deref(), Some("Vendita"));
    assert_eq!(transport.numero_colli, Some(3));
    assert_eq!(transport.descrizione.as_deref(), Some("Colli vari"));
    assert_eq!(transport.unita_misura_peso.as_deref(), Some("KG"));
    assert_eq!(transport.peso_lordo, Some(amount2("12.34")));
    assert_eq!(transport.peso_netto, Some(amount2("10.00")));
    assert_eq!(
        transport.data_ora_ritiro.as_deref(),
        Some("2024-02-28T09:30:00+01:00")
    );
    assert_eq!(
        transport.data_inizio_trasporto,
        Some(CivilDate::from_ymd(2024, 2, 28).unwrap())
    );
    assert_eq!(transport.tipo_resa.as_deref(), Some("DAP"));
    assert!(transport.indirizzo_resa.is_some());
    assert_eq!(
        transport.data_ora_consegna.as_deref(),
        Some("2024-02-29T12:00:00Z")
    );

    let main_invoice = general.fattura_principale.as_ref().unwrap();
    assert_eq!(
        main_invoice.numero_fattura_principale.as_deref(),
        Some("FP-1")
    );
    assert_eq!(
        main_invoice.data_fattura_principale,
        Some(CivilDate::from_ymd(2024, 1, 1).unwrap())
    );
}

#[test]
fn complete_lines_and_summaries() {
    let document = complete_document();
    let body = &document.bodies[0];
    let services = body.dati_beni_servizi.as_ref().unwrap();
    assert_eq!(services.dettaglio_linee.len(), 2);
    let line = &services.dettaglio_linee[0];
    assert_eq!(line.numero_linea, Some(1));
    assert_eq!(line.tipo_cessione_prestazione.as_deref(), Some("SC"));
    assert_eq!(line.codice_articolo.len(), 2);
    assert_eq!(
        line.codice_articolo[0].codice_tipo.as_deref(),
        Some("TARIC")
    );
    assert_eq!(
        line.codice_articolo[1].codice_valore.as_deref(),
        Some("12345678")
    );
    assert_eq!(line.descrizione.as_deref(), Some("Servizio & consulenza"));
    assert_eq!(line.quantita, Some(amount("2.50000000")));
    assert_eq!(line.unita_misura.as_deref(), Some("NR"));
    assert_eq!(
        line.data_inizio_periodo,
        Some(CivilDate::from_ymd(2024, 1, 1).unwrap())
    );
    assert_eq!(
        line.data_fine_periodo,
        Some(CivilDate::from_ymd(2024, 1, 31).unwrap())
    );
    assert_eq!(line.prezzo_unitario, Some(amount("12.34567890")));
    assert_eq!(line.sconto_maggiorazione.len(), 2);
    assert_eq!(
        line.sconto_maggiorazione[1].importo,
        Some(amount("0.50000000"))
    );
    assert_eq!(line.prezzo_totale, Some(amount("27.50000000")));
    assert_eq!(line.aliquota_iva, Some(rate("22.00")));
    assert_eq!(line.ritenuta.as_deref(), Some("SI"));
    assert_eq!(line.natura.as_deref(), Some("N1"));
    assert_eq!(line.riferimento_amministrazione.as_deref(), Some("UFF-3"));
    let other = &line.altri_dati_gestionali[0];
    assert_eq!(other.tipo_dato.as_deref(), Some("INTENTO"));
    assert_eq!(other.riferimento_testo.as_deref(), Some("Rif. testo"));
    assert_eq!(other.riferimento_numero, Some(amount("99.12345678")));
    assert_eq!(
        other.riferimento_data,
        Some(CivilDate::from_ymd(2024, 3, 1).unwrap())
    );

    assert_eq!(services.dati_riepilogo.len(), 2);
    let summary = &services.dati_riepilogo[0];
    assert_eq!(summary.aliquota_iva, Some(rate("22.00")));
    assert_eq!(summary.natura.as_deref(), Some("N1"));
    assert_eq!(summary.spese_accessorie, Some(amount2("1.00")));
    assert_eq!(summary.arrotondamento, Some(amount("0.01000000")));
    assert_eq!(summary.imponibile_importo, Some(amount2("27.50")));
    assert_eq!(summary.imposta, Some(amount2("6.05")));
    assert_eq!(summary.esigibilita_iva.as_deref(), Some("I"));
    assert_eq!(summary.riferimento_normativo.as_deref(), Some("Art. 1"));
}

#[test]
fn complete_payments_attachments_and_unknown() {
    let document = complete_document();
    let body = &document.bodies[0];
    let vehicle = body.dati_veicoli.as_ref().unwrap();
    assert_eq!(vehicle.data, Some(CivilDate::from_ymd(2024, 3, 2).unwrap()));
    assert_eq!(vehicle.totale_percorso.as_deref(), Some("120"));

    assert_eq!(body.dati_pagamento.len(), 1);
    let payment = &body.dati_pagamento[0];
    assert_eq!(payment.condizioni_pagamento.as_deref(), Some("TP02"));
    let detail = &payment.dettaglio_pagamento[0];
    assert_eq!(detail.beneficiario.as_deref(), Some("Beneficiario S.r.l."));
    assert_eq!(detail.modalita_pagamento.as_deref(), Some("MP05"));
    assert_eq!(
        detail.data_riferimento_termini_pagamento,
        Some(CivilDate::from_ymd(2024, 3, 1).unwrap())
    );
    assert_eq!(detail.giorni_termini_pagamento, Some(30));
    assert_eq!(
        detail.data_scadenza_pagamento,
        Some(CivilDate::from_ymd(2024, 3, 31).unwrap())
    );
    assert_eq!(detail.importo_pagamento, Some(amount2("122.00")));
    assert_eq!(detail.cod_ufficio_postale.as_deref(), Some("00100"));
    assert_eq!(detail.cognome_quietanzante.as_deref(), Some("Verdi"));
    assert_eq!(detail.nome_quietanzante.as_deref(), Some("Anna"));
    assert_eq!(detail.cf_quietanzante.as_deref(), Some("RSSMRA80A01H501U"));
    assert_eq!(detail.titolo_quietanzante.as_deref(), Some("Sig.ra"));
    assert_eq!(
        detail.istituto_finanziario.as_deref(),
        Some("Banca di prova")
    );
    assert_eq!(detail.iban.as_deref(), Some("IT60X0542811101000000123456"));
    assert_eq!(detail.abi.as_deref(), Some("05428"));
    assert_eq!(detail.cab.as_deref(), Some("11101"));
    assert_eq!(detail.bic.as_deref(), Some("BPMOIT22"));
    assert_eq!(detail.sconto_pagamento_anticipato, Some(amount2("1.00")));
    assert_eq!(
        detail.data_limite_pagamento_anticipato,
        Some(CivilDate::from_ymd(2024, 3, 10).unwrap())
    );
    assert_eq!(detail.penalita_pagamenti_ritardati, Some(amount2("2.00")));
    assert_eq!(
        detail.data_decorrenza_penale,
        Some(CivilDate::from_ymd(2024, 4, 1).unwrap())
    );
    assert_eq!(detail.codice_pagamento.as_deref(), Some("PAG-1"));

    assert_eq!(body.allegati.len(), 1);
    let attachment = &body.allegati[0];
    assert_eq!(attachment.nome_attachment.as_deref(), Some("allegato.txt"));
    assert_eq!(attachment.algoritmo_compressione.as_deref(), Some("ZIP"));
    assert_eq!(attachment.formato_attachment.as_deref(), Some("TXT"));
    assert_eq!(
        attachment.descrizione_attachment.as_deref(),
        Some("Allegato di prova")
    );
    assert_eq!(attachment.attachment.as_deref(), Some("SGVsbG8="));

    assert_eq!(document.unknown.len(), 1);
    let unknown = document.unknown[0];
    assert_eq!(unknown.name().local(), b"ElementoSconosciuto");
    let source = include_bytes!("data/model/complete.xml");
    let needle = b"<ElementoSconosciuto";
    let expected = source
        .windows(needle.len())
        .position(|window| window == needle)
        .unwrap();
    assert_eq!(usize::try_from(unknown.offset()).unwrap(), expected);
    assert_eq!(unknown.depth(), 3);
}

#[test]
fn parses_the_lotto_fixture_in_order() {
    let document = parse(include_bytes!("data/model/lotto.xml")).unwrap();
    assert_eq!(document.bodies.len(), 2);
    let numbers: Vec<&str> = document
        .bodies
        .iter()
        .map(|body| {
            body.dati_generali
                .as_ref()
                .unwrap()
                .dati_generali_documento
                .as_ref()
                .unwrap()
                .numero
                .as_deref()
                .unwrap()
        })
        .collect();
    assert_eq!(numbers, ["BODY-1", "BODY-2"]);
    assert_eq!(
        document.bodies[0]
            .dati_beni_servizi
            .as_ref()
            .unwrap()
            .dettaglio_linee
            .len(),
        1
    );
    assert_eq!(
        document.bodies[1]
            .dati_beni_servizi
            .as_ref()
            .unwrap()
            .dettaglio_linee
            .len(),
        2
    );
}

#[test]
fn missing_root_and_leading_whitespace() {
    for input in [&b""[..], b"   \n\t", b"<?xml version=\"1.0\"?>"] {
        assert!(matches!(parse(input), Err(ModelError::MissingRoot)));
    }
    let document = parse(b"  \n<FatturaElettronica versione=\"FPR12\"/>").unwrap();
    assert_eq!(document.versione.as_deref(), Some("FPR12"));
    assert!(document.bodies.is_empty());
}

#[test]
fn rejects_an_unsupported_root() {
    let error = parse(b"<FatturaElettronicaSemplificata versione=\"FSM10\"/>").unwrap_err();
    assert!(matches!(error, ModelError::UnsupportedRoot));
}

#[test]
fn accepts_a_prefixed_root_and_ignores_foreign_attributes() {
    let document = parse(
        br#"<p:FatturaElettronica xmlns:p="urn:x" xmlns:xsi="urn:y" xsi:schemaLocation="s" altro="z" versione="FPR12" SistemaEmittente="X"/>"#,
    )
    .unwrap();
    assert_eq!(document.versione.as_deref(), Some("FPR12"));
    assert_eq!(document.sistema_emittente.as_deref(), Some("X"));
}

#[test]
fn self_closing_children_are_present_but_empty() {
    let document = parse(
        br#"<FatturaElettronica versione="FPR12"><FatturaElettronicaHeader><SoggettoEmittente/></FatturaElettronicaHeader><FatturaElettronicaBody/></FatturaElettronica>"#,
    )
    .unwrap();
    let header = document.header.as_ref().unwrap();
    assert!(header.dati_trasmissione.is_none());
    assert_eq!(header.soggetto_emittente.as_deref(), Some(""));
    assert_eq!(document.bodies.len(), 1);
    assert!(document.bodies[0].dati_generali.is_none());
    assert!(document.unknown.is_empty());
}

#[test]
fn cdata_and_split_text_runs_are_joined() {
    let document = parse(
        br#"<FatturaElettronica versione="FPR12"><FatturaElettronicaBody><DatiBeniServizi><DettaglioLinee><Descrizione><![CDATA[a]]>b<![CDATA[c]]></Descrizione></DettaglioLinee></DatiBeniServizi></FatturaElettronicaBody></FatturaElettronica>"#,
    )
    .unwrap();
    assert_eq!(
        document.bodies[0]
            .dati_beni_servizi
            .as_ref()
            .unwrap()
            .dettaglio_linee[0]
            .descrizione
            .as_deref(),
        Some("abc")
    );
}

#[test]
fn predefined_and_character_entities_are_decoded() {
    let document = parse(
        br#"<FatturaElettronica versione="FPR12"><FatturaElettronicaBody><DatiBeniServizi><DettaglioLinee><Descrizione>&amp;&lt;&gt;&apos;&quot;</Descrizione><UnitaMisura>x&#65;y&#x42;z</UnitaMisura></DettaglioLinee></DatiBeniServizi></FatturaElettronicaBody></FatturaElettronica>"#,
    )
    .unwrap();
    let line = &document.bodies[0]
        .dati_beni_servizi
        .as_ref()
        .unwrap()
        .dettaglio_linee[0];
    assert_eq!(line.descrizione.as_deref(), Some("&<>'\""));
    assert_eq!(line.unita_misura.as_deref(), Some("xAyBz"));
}

#[test]
fn undefined_entities_are_typed_errors() {
    let named = parse(
        br#"<FatturaElettronica versione="FPR12"><FatturaElettronicaBody><DatiBeniServizi><DettaglioLinee><Descrizione>&nope;</Descrizione></DettaglioLinee></DatiBeniServizi></FatturaElettronicaBody></FatturaElettronica>"#,
    );
    assert!(matches!(named, Err(ModelError::UndefinedEntity)));

    let numeric = parse(
        br#"<FatturaElettronica versione="FPR12"><FatturaElettronicaBody><DatiBeniServizi><DettaglioLinee><Descrizione>&#xZZ;</Descrizione></DettaglioLinee></DatiBeniServizi></FatturaElettronicaBody></FatturaElettronica>"#,
    );
    assert!(matches!(numeric, Err(ModelError::UndefinedEntity)));
}

#[test]
fn invalid_numeric_and_date_values_become_none() {
    let document = parse(
        br#"<FatturaElettronica versione="FPR12"><FatturaElettronicaBody>
<DatiGenerali><DatiGeneraliDocumento><Data>2023-02-29</Data></DatiGeneraliDocumento>
<DatiOrdineAcquisto><RiferimentoNumeroLinea>abc</RiferimentoNumeroLinea></DatiOrdineAcquisto>
<DatiDDT><RiferimentoNumeroLinea>abc</RiferimentoNumeroLinea></DatiDDT></DatiGenerali>
<DatiBeniServizi><DettaglioLinee><Descrizione>x</Descrizione><PrezzoUnitario>abc</PrezzoUnitario><AliquotaIVA>101.00</AliquotaIVA></DettaglioLinee></DatiBeniServizi>
</FatturaElettronicaBody></FatturaElettronica>"#,
    )
    .unwrap();
    let general = document.bodies[0].dati_generali.as_ref().unwrap();
    assert_eq!(general.dati_generali_documento.as_ref().unwrap().data, None);
    assert!(
        general.dati_ordine_acquisto[0]
            .riferimento_numero_linea
            .is_empty()
    );
    let line = &document.bodies[0]
        .dati_beni_servizi
        .as_ref()
        .unwrap()
        .dettaglio_linee[0];
    assert_eq!(line.prezzo_unitario, None);
    assert_eq!(line.aliquota_iva, None);
}

#[test]
fn element_inside_a_text_leaf_is_rejected() {
    let error = parse(
        br#"<FatturaElettronica versione="FPR12"><FatturaElettronicaBody><DatiBeniServizi><DettaglioLinee><Descrizione><b/></Descrizione></DettaglioLinee></DatiBeniServizi></FatturaElettronicaBody></FatturaElettronica>"#,
    )
    .unwrap_err();
    assert!(matches!(error, ModelError::UnexpectedElement));
}

#[test]
fn truncated_and_malformed_input_is_a_typed_error() {
    let truncated = parse(
        br#"<FatturaElettronica versione="FPR12"><FatturaElettronicaHeader><DatiTrasmissione>"#,
    )
    .unwrap_err();
    assert!(matches!(truncated, ModelError::Xml(_)));
    assert!(std::error::Error::source(&truncated).is_some());

    let malformed = parse(b"<").unwrap_err();
    assert!(matches!(malformed, ModelError::Xml(_)));
}

#[test]
fn invalid_utf8_in_a_text_leaf_is_rejected() {
    let mut input = Vec::from(
        &b"<FatturaElettronica versione=\"FPR12\"><FatturaElettronicaBody><DatiBeniServizi><DettaglioLinee><Descrizione>"[..],
    );
    input.push(0xff);
    input.extend_from_slice(
        b"</Descrizione></DettaglioLinee></DatiBeniServizi></FatturaElettronicaBody></FatturaElettronica>",
    );
    let error = parse(&input).unwrap_err();
    assert!(matches!(error, ModelError::InvalidUtf8));
}

#[test]
fn unknown_root_and_nested_elements_are_recorded_and_skipped() {
    let document = parse(
        br#"<FatturaElettronica versione="FPR12"><Sconosciuto><Figlio/></Sconosciuto><FatturaElettronicaBody><Altro/></FatturaElettronicaBody></FatturaElettronica>"#,
    )
    .unwrap();
    assert_eq!(document.unknown.len(), 2);
    assert_eq!(document.unknown[0].name().local(), b"Sconosciuto");
    assert_eq!(document.unknown[0].depth(), 2);
    assert_eq!(document.unknown[1].name().local(), b"Altro");
    assert_eq!(document.bodies.len(), 1);
}

#[test]
fn model_errors_render_and_expose_their_source() {
    let errors = [
        ModelError::MissingRoot,
        ModelError::UnsupportedRoot,
        ModelError::UnexpectedElement,
        ModelError::UnexpectedEof,
        ModelError::UndefinedEntity,
        ModelError::InvalidUtf8,
    ];
    for error in errors {
        assert!(!error.to_string().is_empty());
        assert!(std::error::Error::source(&error).is_none());
    }
    let xml = parse(b"<").unwrap_err();
    assert!(xml.to_string().contains("malformed XML"));
    assert!(std::error::Error::source(&xml).is_some());
}

#[test]
fn self_closing_text_leaf_is_present_and_empty() {
    let document = parse(
        br#"<FatturaElettronica versione="FPR12"><FatturaElettronicaBody><DatiGenerali><DatiGeneraliDocumento><Divisa/></DatiGeneraliDocumento></DatiGenerali></FatturaElettronicaBody></FatturaElettronica>"#,
    )
    .unwrap();
    assert_eq!(
        document.bodies[0]
            .dati_generali
            .as_ref()
            .unwrap()
            .dati_generali_documento
            .as_ref()
            .unwrap()
            .divisa
            .as_deref(),
        Some("")
    );
}

#[test]
fn unknown_elements_are_retained_in_every_container() {
    let document = parse(include_bytes!("data/model/unknown_everywhere.xml")).unwrap();
    // One unknown child per complex container, plus the unknown root child.
    assert_eq!(document.unknown.len(), 36);
    assert_eq!(document.unknown[0].name().local(), b"IgnotoRadice");
    let source = include_bytes!("data/model/unknown_everywhere.xml");
    let needle = b"<IgnotoRadice";
    let expected = source
        .windows(needle.len())
        .position(|window| window == needle)
        .unwrap();
    assert_eq!(
        usize::try_from(document.unknown[0].offset()).unwrap(),
        expected
    );
    assert_eq!(document.unknown[0].depth(), 2);
    assert_eq!(document.bodies.len(), 1);
    let names: Vec<&[u8]> = document
        .unknown
        .iter()
        .map(|unknown| unknown.name().local())
        .collect();
    for expected in [
        b"IgnotoHeader".as_slice(),
        b"IgnotoTrasmissione".as_slice(),
        b"IgnotoId".as_slice(),
        b"IgnotoContatti".as_slice(),
        b"IgnotoCedente".as_slice(),
        b"IgnotoDatiAnagrafici".as_slice(),
        b"IgnotoAnagrafica".as_slice(),
        b"IgnotoIndirizzo".as_slice(),
        b"IgnotoRea".as_slice(),
        b"IgnotoRappresentante".as_slice(),
        b"IgnotoCessionario".as_slice(),
        b"IgnotoRappresentanteCessionario".as_slice(),
        b"IgnotoTerzo".as_slice(),
        b"IgnotoBody".as_slice(),
        b"IgnotoDatiGenerali".as_slice(),
        b"IgnotoDocumento".as_slice(),
        b"IgnotoRitenuta".as_slice(),
        b"IgnotoBollo".as_slice(),
        b"IgnotoCassa".as_slice(),
        b"IgnotoSconto".as_slice(),
        b"IgnotoCorrelati".as_slice(),
        b"IgnotoSal".as_slice(),
        b"IgnotoDdt".as_slice(),
        b"IgnotoTrasporto".as_slice(),
        b"IgnotoVettore".as_slice(),
        b"IgnotoFatturaPrincipale".as_slice(),
        b"IgnotoBeniServizi".as_slice(),
        b"IgnotoLinea".as_slice(),
        b"IgnotoCodiceArticolo".as_slice(),
        b"IgnotoAltriDati".as_slice(),
        b"IgnotoRiepilogo".as_slice(),
        b"IgnotoVeicoli".as_slice(),
        b"IgnotoPagamento".as_slice(),
        b"IgnotoDettaglioPagamento".as_slice(),
        b"IgnotoAllegati".as_slice(),
    ] {
        assert!(names.contains(&expected), "{expected:?}");
    }
}

#[test]
fn document_value_stays_small_enough_to_return_by_value() {
    let size = std::mem::size_of::<Document<'static>>();
    // E2.2 acceptance requires the size to be measured. The bound catches an
    // accidental inline explosion (for example a second inline body slot); it
    // is not a performance target.
    assert!(size <= 16_384, "Document is {size} bytes");
    assert_eq!(std::mem::size_of::<Document<'static>>(), size);
    println!("Document<'_>: {size} bytes");
}
