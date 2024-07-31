use image::{imageops, ImageFormat};
use magpie_engine::Temple;
use std::io::Cursor;

use crate::{debug, get_portrait, resize_img, Card};

pub fn gen_portrait(card: &Card) -> Vec<u8> {
    match card.set.code() {
        "aug" => gen_aug_portrait(card),
        "com" | "ete" | "egg" => gen_imf_portrait(card),
        _ => unimplemented!(),
    }
}

fn gen_imf_portrait(card: &Card) -> Vec<u8> {
    resize_img(get_portrait(&card.portrait), 2)
}

fn gen_aug_portrait(card: &Card) -> Vec<u8> {
    let portrait =
        image::load(Cursor::new(get_portrait(&card.portrait)), ImageFormat::Png).unwrap();

    let bg = &format!(
        "https://raw.githubusercontent.com/answearingmachine/card-printer/main/dist/printer/assets/bg/bg_{}_{}.png",

        match card.rarity.to_string().as_str(){
            "Common" | "Uncommon" | "Side" => "common",
            "Rare" | "Unique" => "rare",
            r => unreachable!("{}", r)
        },
        if let Some(t) = Temple::from(card.temple).flags().next() {
            match *t {
                Temple::BEAST => "beast",
                Temple::UNDEAD => "undead",
                Temple::TECH => "tech",
                Temple::MAGICK => "magick",
                _ => unreachable!(),
            }
        } else {
            unreachable!()
        },
    );

    debug!(bg);

    let mut bg = image::load(Cursor::new(get_portrait(bg)), ImageFormat::Png).unwrap();

    imageops::overlay(&mut bg, &portrait, 0, 0);

    let mut out = vec![];
    bg.write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .unwrap();

    resize_img(out, 2)
}