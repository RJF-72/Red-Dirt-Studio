#!/usr/bin/env python3
"""
RDRS Dataset Builder - Focused Instrument Selection
Builds aligned SFZ+FLAC dataset for neural timbre training.
Curated for: Fiddle, Pedal Steel, Baritone Acoustic, Acoustic Guitar, Resonator/Dobro,
Country Drums, Worship Drums, Mandolin, Banjo, Harmonica, Keyboard, Piano,
Acoustic Bass, Electric Bass, Alto Sax, Tenor Sax
"""

import os
import json
import re
from pathlib import Path
from dataclasses import dataclass, asdict
from typing import List, Dict, Optional, Set
import soundfile as sf
import numpy as np

RDRS_ROOT = Path(r"E:\Red_Dirt_Rhythm\red_dirt_rhythm_studio\RDRS")
OUTPUT_DIR = Path(r"E:\Red_Dirt_Rhythm\red_dirt_rhythm_studio\dataset_output")

# ============================================================
# CURATED INSTRUMENTS - Your Core Country/Americana Band
# ============================================================
STARTER_INSTRUMENTS = {
    # --- FIDDLE ---
    "fiddle": [
        "ethnic_(gm_)_fiddle_americana",
        "ethnic_(gm_)_fiddle_country",
        "ethnic_(gm_)_fiddle_country_folk",
        "ethnic_(gm_)_fiddle_americana_folk_string",
        "ethnic_(gm_)_fiddle_americana_mountain_string",
        "ethnic_(gm_)_fiddle_americana_warm",
        "playable_realtracks_ethnic_111_fiddle_americana_alt",
        "playable_realtracks_ethnic_111_fiddle_country_alt",
        "playable_realtracks_ethnic_111_fiddle_country_folk_alt",
        "playable_realtracks_ethnic_111_fiddle_americana_warm_alt",
        "playable_realtracks_ethnic_111_fiddle_americana_folk_5_string_alt",
        "playable_realtracks_ethnic_111_fiddle_americana_mountain_5_string_alt",
        "playable_realtracks_ethnic_111_fiddle_americana_pizzicato_5_string_alt",
    ],

    # --- PEDAL STEEL GUITAR ---
    "pedal_steel": [
        "electric_guitars_etc_clean_(gm_)_pedal_steel_clean_country",
        "electric_guitars_etc_clean_(gm_)_pedal_steel_clean_vintage_country",
        "electric_guitars_etc_clean_(gm_)_pedal_steel_clean_delay_lush",
        "electric_guitars_etc_clean_(gm_)_pedal_steel_clean_cajun_zydeco",
        "electric_guitars_etc_clean_(gm_)_pedal_steel_clean_dreamy",
        "electric_guitars_etc_clean_(gm_)_pedal_steel_clean_west_coast_ballad",
        "playable_realtracks_electric_guitars_etc_clean_028_pedal_steel_clean_country_alt",
        "playable_realtracks_electric_guitars_etc_clean_028_pedal_steel_clean_vintage_country_alt",
        "playable_realtracks_electric_guitars_etc_clean_028_pedal_steel_clean_cajun_zydeco_alt",
        "playable_realtracks_electric_guitars_etc_clean_028_pedal_steel_clean_delay_lush_alt",
        "playable_realtracks_electric_guitars_etc_clean_028_pedal_steel_clean_dreamy_alt",
        "playable_realtracks_electric_guitars_etc_clean_028_pedal_steel_clean_west_coast_ballad_alt",
        # Distorted pedal steel
        "playable_realtracks_electric_guitars_etc_distorted_031_pedal_steel_distortion_ambient_delay_alt",
        "playable_realtracks_electric_guitars_etc_distorted_031_pedal_steel_distortion_country_pop_alt",
        "playable_realtracks_electric_guitars_etc_distorted_031_pedal_steel_distortion_country_rock_alt",
        "playable_realtracks_electric_guitars_etc_distorted_031_pedal_steel_distortion_crunch_delay_alt",
        "playable_realtracks_electric_guitars_etc_distorted_031_pedal_Ankle_steel_distortion_rock_gritty_delay_alt",
    ],

    # --- BARITONE ACOUSTIC GUITAR ---
    "baritone_acoustic_guitar": [
        "acoustic_guitars_etc_(gm_)_guitar_baritone_acoustic_americana",
        "acoustic_guitars_etc_(gm_)_guitar_baritone_acoustic_americana_fingerpicking",
        "playable_realtracks_acoustic_guitars_etc_026_guitar_baritone_acoustic_americana_alt",
        "playable_realtracks_acoustic_guitars_etc_026_guitar_baritone_acoustic_americana_fingerpicking_alt",
    ],

    # --- ACOUSTIC GUITAR ---
    "acoustic_guitar published": [
        # Core acoustic guitars
        "guitar_finger_picked_acoustic",
        "guitar_warm_picked_acoustic",
        "finger_picked_acoustic",
        "acoustic_guitars_etc_(gm_)_guitar_acoustic_campfire",
        "acoustic_guitars_etc_(gm_)_guitar_acoustic_folk",
        "acoustic_guitars_etc_(gm_)_guitar_acoustic_bright_songwriter_fingerpicking",
        "acoustic_guitars_etc_(gm_)_guitar_acoustic_americana",
        "acoustic_guitars_etc_(gm_)_guitar_acoustic_modern_folk",
        "acoustic_guitars_etc_(gm_)_guitar_acoustic_natural_pop_folk",
        "acoustic_guitars_etc_(gm_)_guitar_acoustic_songwriter_fingerpicking",
        "acoustic_guitars_etc_(gm_)_guitar_acoustic_songwriter_gentle",
        "acoustic_guitars_etc_(gm_)_guitar_acoustic_bright_celtic",
        "acoustic_guitars_etc_(gm_)_guitar_acoustic_bright_pop",
        # Playable realtracks
        "playable_realtracks_acoustic_guitars_etc_0
    ],