#include "rptadvdcs/rptadvdcs.h"

#include <assert.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

int main(void) {
  const struct rptadv_dcs_descriptor *api = rptadv_dcs_descriptor();
  assert(api == rptadv_dcs_descriptor());
  assert(api->struct_size == sizeof(*api));
  assert(api->abi_version == RPTADV_DCS_ABI_VERSION);
  assert(strcmp(api->capability_name, "rptadv.dcs") == 0);
  assert(api->code_supported && api->configured_code && api->receive_create &&
         api->receive_destroy && api->receive_configure && api->receive_valid &&
         api->receive_enabled && api->receive_process && api->transmit_create &&
         api->transmit_destroy && api->transmit_configure &&
         api->transmit_update_config_from && api->transmit_generate);
  assert(api->code_supported(023) == 1);
  assert(api->code_supported(-1) == 0);
  assert(api->configured_code(0777) == 0777);
  assert(api->configured_code(01000) == -1);

  struct rptadv_dcs_receive *rx = NULL;
  struct rptadv_dcs_transmit *whole = NULL;
  struct rptadv_dcs_transmit *split = NULL;
  struct rptadv_dcs_transmit *reference_new = NULL;
  struct rptadv_dcs_transmit *prepared = NULL;
  assert(api->receive_create(&rx) == RPTADV_DCS_OK && rx);
  assert(api->transmit_create(&whole) == RPTADV_DCS_OK && whole);
  assert(api->transmit_create(&split) == RPTADV_DCS_OK && split);
  assert(api->transmit_create(&reference_new) == RPTADV_DCS_OK && reference_new);
  assert(api->transmit_create(&prepared) == RPTADV_DCS_OK && prepared);
  assert(api->receive_configure(rx, 023, 0) == RPTADV_DCS_OK);
  assert(api->receive_enabled(rx) == 1);
  assert(api->transmit_configure(whole, 023, 0) == RPTADV_DCS_OK);
  assert(api->transmit_configure(split, 023, 0) == RPTADV_DCS_OK);
  assert(api->transmit_configure(reference_new, 025, 1) == RPTADV_DCS_OK);
  assert(api->transmit_configure(prepared, 025, 1) == RPTADV_DCS_OK);

  float expected[9217];
  float actual[9217];
  assert(api->transmit_generate(whole, expected, 9217, 0.25f, 1, 0) ==
         RPTADV_DCS_OK);
  for (uint32_t offset = 0; offset < 9217;) {
    uint32_t count = 791 < 9217 - offset ? 791 : 9217 - offset;
    assert(api->transmit_generate(split, actual + offset, count, 0.25f, 1, 0) ==
           RPTADV_DCS_OK);
    offset += count;
  }
  assert(memcmp(expected, actual, sizeof(expected)) == 0);
  assert(expected[0] == 0.25f);
  assert(api->transmit_generate(reference_new, expected, 9217, 0.25f, 1, 0) ==
         RPTADV_DCS_OK);
  assert(api->transmit_update_config_from(split, prepared) == RPTADV_DCS_OK);
  assert(api->transmit_generate(reference_new, expected, 9217, 0.25f, 1, 0) ==
         RPTADV_DCS_OK);
  assert(api->transmit_generate(split, actual, 9217, 0.25f, 1, 0) ==
         RPTADV_DCS_OK);
  assert(memcmp(expected, actual, sizeof(expected)) == 0);
  assert(api->transmit_generate(whole, actual, 9217, 0.25f, 1, 0) ==
         RPTADV_DCS_OK);
  assert(memcmp(expected, actual, sizeof(expected)) != 0);

  float stereo[48000 * 2];
  const uint32_t wire_word = 0x763813;
  for (uint32_t i = 0; i < 48000; ++i) {
    uint32_t bit = ((uint64_t)i * 1344 / 480000) % 23;
    float sample = (wire_word & (1U << bit)) ? 0.375f : -0.375f;
    stereo[2 * i] = sample;
    stereo[2 * i + 1] = -sample;
  }
  uint32_t valid = 0;
  for (uint32_t offset = 0; offset < 48000;) {
    uint32_t count = 257 < 48000 - offset ? 257 : 48000 - offset;
    assert(api->receive_process(rx, stereo + 2 * offset, count, &valid) ==
           RPTADV_DCS_OK);
    offset += count;
  }
  assert(valid == 1 && api->receive_valid(rx) == 1);
  assert(api->receive_configure(rx, 01000, 0) == RPTADV_DCS_OK);
  assert(api->receive_enabled(rx) == 0 && api->receive_valid(rx) == 0);

  assert(api->receive_create(NULL) == RPTADV_DCS_INVALID_ARGUMENT);
  assert(api->transmit_create(NULL) == RPTADV_DCS_INVALID_ARGUMENT);
  assert(api->receive_configure(NULL, 023, 0) == RPTADV_DCS_INVALID_ARGUMENT);
  assert(api->transmit_configure(NULL, 023, 0) == RPTADV_DCS_INVALID_ARGUMENT);
  assert(api->receive_valid(NULL) == 0 && api->receive_enabled(NULL) == 0);
  assert(api->receive_process(rx, NULL, 1, &valid) ==
         RPTADV_DCS_INVALID_ARGUMENT);
  assert(api->receive_process(rx, stereo, 1, NULL) ==
         RPTADV_DCS_INVALID_ARGUMENT);
  assert(api->receive_process(NULL, stereo, 1, &valid) ==
         RPTADV_DCS_INVALID_ARGUMENT);
  assert(api->receive_process(rx, NULL, 0, &valid) == RPTADV_DCS_OK);
  assert(api->transmit_generate(split, NULL, 1, 0.25f, 1, 0) ==
         RPTADV_DCS_INVALID_ARGUMENT);
  assert(api->transmit_generate(NULL, actual, 1, 0.25f, 1, 0) ==
         RPTADV_DCS_INVALID_ARGUMENT);
  assert(api->transmit_generate(split, NULL, 0, 0.25f, 1, 0) == RPTADV_DCS_OK);
  assert(api->transmit_update_config_from(split, NULL) ==
         RPTADV_DCS_INVALID_ARGUMENT);
  assert(api->transmit_update_config_from(NULL, prepared) ==
         RPTADV_DCS_INVALID_ARGUMENT);
  assert(api->transmit_update_config_from(split, split) ==
         RPTADV_DCS_INVALID_ARGUMENT);

  api->receive_destroy(rx);
  api->transmit_destroy(whole);
  api->transmit_destroy(split);
  api->transmit_destroy(reference_new);
  api->transmit_destroy(prepared);
  api->receive_destroy(NULL);
  api->transmit_destroy(NULL);
  return 0;
}
