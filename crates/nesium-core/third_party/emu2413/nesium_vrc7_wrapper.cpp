#include "emu2413.h"

extern "C" {
OPLL* nesium_vrc7_opll_new(unsigned int clk, unsigned int rate) {
    return OPLL_new(clk, rate);
}

void nesium_vrc7_opll_delete(OPLL* opll) {
    OPLL_delete(opll);
}

void nesium_vrc7_opll_reset(OPLL* opll) {
    OPLL_reset(opll);
}

void nesium_vrc7_opll_reset_patch(OPLL* opll, unsigned char patch_type) {
    OPLL_resetPatch(opll, patch_type);
}

void nesium_vrc7_opll_set_chip_type(OPLL* opll, unsigned char chip_type) {
    OPLL_setChipType(opll, chip_type);
}

void nesium_vrc7_opll_write_reg(OPLL* opll, unsigned int reg, unsigned char value) {
    OPLL_writeReg(opll, reg, value);
}

short nesium_vrc7_opll_calc(OPLL* opll) {
    return OPLL_calc(opll);
}
}
