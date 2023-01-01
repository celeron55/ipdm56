/*
ipdmsw - iPDM56 firmware template
Copyright (c) 2023 Perttu "celeron55" Ahola

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

// MODULE_DEF(name, timeout_ms)
// PARAM_DEF(module_name, name, type, default_value, report_hysteresis)
// report_hysteresis=0: no reporting

// Example module for which we receive a parameter
MODULE_DEF(example_module, 5000);
PARAM_DEF(example_module, foobar, uint8_t, 0, 1);

// NOTE: While there are bunches of example values being received from the
//       example modules, nothing is actually sent to them to control them, and
//       power distribution isn't set up either. You will have to do that
//       according to your specific setup.

// Example: Outlander OBC
MODULE_DEF(obc, 5000);
PARAM_DEF(obc, battery_12v_voltage, uint16_t, 0, 10); // 0.01V/bit
PARAM_DEF(obc, dcdc_status, uint8_t, 0, 1);
PARAM_DEF(obc, battery_voltage, uint16_t, 0, 2);
PARAM_DEF(obc, supply_voltage, uint8_t, 0, 5);
PARAM_DEF(obc, supply_current, uint8_t, 0, 5); // 0.1A/bit
PARAM_DEF(obc, evse_pwm, uint8_t, 0, 1); // 100 = 100%

// Example: Outlander CAN-controlled heater
MODULE_DEF(heater, 5000);
PARAM_DEF(heater, heating, bool, false, 1);
PARAM_DEF(heater, hv_present, bool, false, 1);
PARAM_DEF(heater, temperature, int8_t, 127, 2);

