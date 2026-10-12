/** @file
 * @brief Native 48 kHz DCS receive and transmit descriptor ABI.
 */
#ifndef RPTADV_DCS_H
#define RPTADV_DCS_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define RPTADV_DCS_ABI_VERSION 1U /**< Initial DCS descriptor ABI. */
#define RPTADV_DCS_OK 0 /**< Operation completed. */
#define RPTADV_DCS_INVALID_ARGUMENT -1 /**< Null required argument or handle. */

/** @brief Opaque state for one DCS receive stream. */
struct rptadv_dcs_receive;
/** @brief Opaque state for one DCS transmit stream or prepared configuration. */
struct rptadv_dcs_transmit;

/** @brief Immutable function table for the process lifetime. */
struct rptadv_dcs_descriptor {
  uint32_t struct_size; /**< Size of this descriptor. */
  uint32_t abi_version; /**< Must equal RPTADV_DCS_ABI_VERSION. */
  const char *capability_name; /**< Stable `rptadv.dcs` identity. */

  /** @brief Return 1 for a supported numeric DCS code, otherwise 0. */
  uint32_t (*code_supported)(int32_t code);
  /** @brief Return the code or -1 when unsupported. */
  int32_t (*configured_code)(int32_t code);

  /** @brief Allocate receive state during setup; out must be nonnull. */
  int32_t (*receive_create)(struct rptadv_dcs_receive **out);
  /** @brief Release receive state after callback use stops; null is accepted. */
  void (*receive_destroy)(struct rptadv_dcs_receive *handle);
  /** @brief Set code and polarity, resetting decode qualification. */
  int32_t (*receive_configure)(struct rptadv_dcs_receive *handle, int32_t code,
                               uint32_t inverted);
  /** @brief Return 1 when the receive code is qualified. */
  uint32_t (*receive_valid)(const struct rptadv_dcs_receive *handle);
  /** @brief Return 1 when a supported receive code is configured. */
  uint32_t (*receive_enabled)(const struct rptadv_dcs_receive *handle);
  /** @brief Process frames of interleaved stereo f32 PCM using the left channel.
   * stereo is readable for 2*frames samples; null is allowed only for zero frames.
   * valid_out receives the current qualification. Processing requires exclusive
   * handle ownership and performs no allocation, lock, logging, or I/O.
   */
  int32_t (*receive_process)(struct rptadv_dcs_receive *handle,
                             const float *stereo, uint32_t frames,
                             uint32_t *valid_out);

  /** @brief Allocate transmit state during setup; out must be nonnull. */
  int32_t (*transmit_create)(struct rptadv_dcs_transmit **out);
  /** @brief Release transmit state after callback use stops; null is accepted. */
  void (*transmit_destroy)(struct rptadv_dcs_transmit *handle);
  /** @brief Set code and polarity, resetting the fractional bit accumulator. */
  int32_t (*transmit_configure)(struct rptadv_dcs_transmit *handle, int32_t code,
                                uint32_t inverted);
  /** @brief Copy prepared code and polarity without resetting phase, bit
   * accumulator, or tail oscillator. Handles must be distinct.
   */
  int32_t (*transmit_update_config_from)(struct rptadv_dcs_transmit *handle,
      const struct rptadv_dcs_transmit *prepared);
  /** @brief Generate frames of mono f32 PCM. Output is writable for frames
   * samples; null is allowed only for zero frames. The caller owns the handle
   * exclusively during this allocation-free audio operation.
   */
  int32_t (*transmit_generate)(struct rptadv_dcs_transmit *handle, float *output,
                               uint32_t frames, float peak, uint32_t enabled,
                               uint32_t turnoff);
};

/** @brief Return the immutable descriptor; validate identity, size and ABI. */
const struct rptadv_dcs_descriptor *rptadv_dcs_descriptor(void);

#ifdef __cplusplus
}
#endif

#endif
