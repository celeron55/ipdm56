/*
ipdm - iPDM56 firmware
Copyright (c) 2022 Perttu "celeron55" Ahola

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

#pragma once
#include "../ipdm_version.h"
#include "ipdm_util.h"
#include "Arduino.h"

namespace ipdm
{

//static auto& CONSOLE = Serial;
#define CONSOLE Serial

void pinMode(int pin, uint8_t mode);
void digitalWrite(int pin, bool state);
bool digitalRead(int pin);
void analogWrite(int pin, uint8_t value);
uint16_t analogRead(int pin);

// NOTE: Remember to use ipdm::digitalRead/Write when accessing the high
//       numbered pins, as the Arduino native function does not understand them
//       and does not give out any error either

constexpr int ACTIVATE_5VSW_PIN = 8;
constexpr int VBAT_PIN = A7;

// I/O extender [0]
constexpr int ED0 = 5600;
constexpr int ED1 = 5601;
constexpr int ED2 = 5602;
constexpr int ED3 = 5603;
constexpr int ED4 = 5604;
constexpr int ED5 = 5605;
constexpr int ED6 = 5606;
constexpr int ED7 = 5607;
constexpr int ED8 = 5608;
constexpr int ED9 = 5609;
constexpr int ED10 = 5610;
constexpr int ED11 = 5611;
constexpr int ED12 = 5612;
constexpr int ED13 = 5613;
constexpr int ED14 = 5614;
constexpr int ED15 = 5615;

// I/O extender [1]
constexpr int ED16 = 5616;
constexpr int ED17 = 5617;
constexpr int ED18 = 5618;
constexpr int ED19 = 5619;
constexpr int ED20 = 5620;
constexpr int ED21 = 5621;
constexpr int ED22 = 5622;
constexpr int ED23 = 5623;
constexpr int ED24 = 5624;
constexpr int ED25 = 5625;
constexpr int ED26 = 5626;
constexpr int ED27 = 5627;
constexpr int ED28 = 5628;
constexpr int ED29 = 5629;
constexpr int ED30 = 5630;
constexpr int ED31 = 5631;

#if IPDM_VERSION == 100

// Actual resistors are (18, 10), but the TVS mixes things up on v1.0.
constexpr uint32_t ADC_FACTOR16_VBAT = ADC_FACTOR16_MV_RESISTORS(85, 40);

// Digital outputs
constexpr int LOUT1       =  ED4;
constexpr int LOUT2       =  ED5;
constexpr int LOUT3       =  ED6;
constexpr int LOUT4       =  ED7;
constexpr int LOUT5       =  ED8;
constexpr int LOUT6       =  ED9;
constexpr int HOUT1       = ED10;
constexpr int HOUT2       = ED11;
constexpr int HOUT3       = ED12;
constexpr int HOUT4       = ED13;
constexpr int HOUT5       = ED14;
constexpr int HOUT6       = ED15;
constexpr int CAN1_CS_PIN =   10;
constexpr int CAN2_CS_PIN =    9;

// PWM outputs
constexpr int LPWM1 = 5;
constexpr int LPWM2 = 6;
constexpr int AOUT1 = LPWM1;
constexpr int AOUT2 = LPWM2;
constexpr int LPWM3 = 255;
constexpr int LPWM4 = 255;

// Inputs
constexpr int VUSB_PIN = ED3; 

#elif IPDM_VERSION == 101

constexpr uint32_t ADC_FACTOR16_VBAT = ADC_FACTOR16_MV_RESISTORS(18, 10);

// Digital outputs
constexpr int LOUT1       = ED21;
constexpr int LOUT2       = ED20;
constexpr int LOUT3       = ED19;
constexpr int LOUT4       = ED18;
constexpr int LOUT5       = ED17;
constexpr int LOUT6       = ED16;
constexpr int HOUT1       = ED28;
constexpr int HOUT2       = ED29;
constexpr int HOUT3       = ED30;
constexpr int HOUT4       = ED31;
constexpr int HOUT5       = ED23;
constexpr int HOUT6       = ED22;
constexpr int HOUT7       = ED12;
constexpr int HOUT8       = ED13;
constexpr int HOUT9       = ED14;
constexpr int HOUT10      = ED15;
constexpr int CAN1_CS_PIN =    4;
constexpr int CAN2_CS_PIN =    7;

// PWM outputs
constexpr int LPWM1 = 5;
constexpr int LPWM2 = 6;
constexpr int AOUT1 = LPWM1;
constexpr int AOUT2 = LPWM2;
constexpr int LPWM3 = 9;
constexpr int LPWM4 = 10;

// Inputs
constexpr int VUSB_PIN       =  ED7;
constexpr int LOUT1_MONITOR  = ED26; 
constexpr int LOUT2_MONITOR  = ED27; 
constexpr int HOUT1_MONITOR  = ED24; 
constexpr int HOUT2_MONITOR  = ED25; 
constexpr int HOUT7_MONITOR  =  ED8; 
constexpr int HOUT8_MONITOR  =  ED9; 
constexpr int HOUT9_MONITOR  = ED10; 
constexpr int HOUT10_MONITOR = ED11; 

#else // IPDM_VERSION

static_assert(false, "Unsupported IPDM_VERSION");

#endif

// Called internally by the ipdm library
void io_begin();

} // namespace ipdm
