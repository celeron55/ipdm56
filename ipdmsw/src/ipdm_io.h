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

namespace ipdm
{

//static auto& CONSOLE = Serial;
#define CONSOLE Serial

// NOTE: Remember to use ipdm::digitalRead/Write when accessing the high
//       numbered pins, as the Arduino native function does not understand them
//       and does not give out any error either

constexpr int CAN1_CS_PIN = 10;
constexpr int CAN2_CS_PIN = 9;
constexpr int POWERSW_PIN = 8;
constexpr int VBAT_PIN = A7;

constexpr int ED0 = 5600;
constexpr int ED1 = 5601;
constexpr int ED2 = 5602;
constexpr int ED3 = 5603;

constexpr int LOUT1 = 5604;
constexpr int LOUT2 = 5605;
constexpr int LOUT3 = 5606;
constexpr int LOUT4 = 5607;
constexpr int LOUT5 = 5608;
constexpr int LOUT6 = 5609;

constexpr int HOUT1 = 5610;
constexpr int HOUT2 = 5611;
constexpr int HOUT3 = 5612;
constexpr int HOUT4 = 5613;
constexpr int HOUT5 = 5614;
constexpr int HOUT6 = 5615;

constexpr int AOUT1 = 5;
constexpr int AOUT2 = 6;

constexpr int VUSB_PIN = ED3;

}
