use image::{imageops, ImageFormat};
use magpie_engine::Temple;
use std::io::Cursor;
use std::u8;

use crate::{get_portrait, resize_img, Card};

pub fn gen_portrait(card: &Card) -> Vec<u8> {
    match card.set.code() {
        "aug" => gen_aug_portrait(card),
        "cti" => gen_simple_portrait(card),
        "std" | "ete" | "egg" | "des" => gen_scale_portrait(card, 4),
        code => todo!("portrait for set code is not implemented yet: {code}"),
    }
}

fn gen_scale_portrait(card: &Card, scale: u32) -> Vec<u8> {
    resize_img(&get_portrait(&card.portrait), scale)
}

fn gen_simple_portrait(card: &Card) -> Vec<u8> {
    get_portrait(&card.portrait)
}

fn gen_aug_portrait(card: &Card) -> Vec<u8> {
    let Ok(portrait) = image::load(Cursor::new(get_portrait(&card.portrait)), ImageFormat::Png)
    else {
        return Vec::new();
    };

    let bg = &format!(
        "https://raw.githubusercontent.com/answearingmachine/card-printer/main/dist/printer/assets/bg/bg_{}_{}.png",

        match card.rarity.to_string().as_str(){
            "Common" | "Uncommon" | "Side" => "common",
            "Rare" | "Unique" => "rare",
            r => unreachable!("{}", r)
        },
        if let Some(t) = card.temple.iter().next() {
            match t {
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

    let mut bg = image::load(Cursor::new(get_portrait(bg)), ImageFormat::Png).unwrap();

    imageops::overlay(&mut bg, &portrait, 0, 0);

    let mut out = vec![];
    bg.write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .unwrap();

    resize_img(&out, 2)
}
