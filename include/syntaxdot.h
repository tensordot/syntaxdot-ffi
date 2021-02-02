#ifndef SYNTAXDOT_H
#define SYNTAXDOT_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * <p>
 * A syntaxdot error.
 * </p>
 * <p>
 * If a function was unsuccessful, the <tt>code</tt> will be set to
 * non-zero and NUL-terminated error message will be assigned to
 * <tt>error</tt>. The caller is responsible for deallocating the
 * message with <tt>syntaxdot_free_string</tt>.
 * </p>
 */
typedef struct {
    int code;
    char *error;
} ExternError;

/**
 * <p>
 * A byte buffer.
 * <p>
 * <p>
 * <tt>data</tt> contains a pointer to the buffer, <tt>len</tt> the
 * buffer length. The caller is responsible for deallocating the
 * buffer with <tt>syntaxdot_free_bytebuffer</tt>.
 * </p>
 */
typedef struct {
    int64_t len;
    uint8_t *data;
} ByteBuffer;

/**
 * Load a syntaxdot annotation model.
 *
 * This function, when successful, returns a handle for the loaded model.
 *
 * @param path The path to the model configuration
 * @param err Pointer to an error value.
 * @return The handle for the annotator.
 */
uint64_t syntaxdot_annotator_load(char const *config_path, ExternError *err);

/**
 * Free a syntaxdot annotation model.
 *
 * @param handle The handle of the model to free.
 * @param err Pointer to an error value.
 */
void syntaxdot_annotator_free(uint64_t handle, ExternError *err);

/**
 * <p>
 * Annotate sentences using a model.
 * </p>
 * <p>
 * This function annotates a set of sentences using the model specified by
 * <tt>handle</tt>. The sentences must be provided as serialized protobuf,
 * <tt>sentences_data</tt> must be a pointer to protobuf data with the length
 * <tt>sentences_data_length<tt>.
 * </p>
 *
 * @param handle The handle of the model to annotate with.
 * @param sentences_data Pointer to the protocol buffer data.
 * @param sentences_data_len Length of the protocol buffer data.
 * @param batch_size Model batch size.
 * @param err Pointer to an error value.
 * @return Buffer with the annotations serialized to protobuf.
 */
ByteBuffer syntaxdot_annotator_annotate(uint64_t handle, uint8_t *sentences_data,
                                        int32_t sentences_data_len, size_t batch_size,
                                        ExternError *err);

/**
 * Set the number of Torch inter-op threads.
 */
void syntaxdot_set_num_intraop_threads(int32_t n_threads);

/**
 * Set the number of Torch inter-op threads.
 */
void syntaxdot_set_num_intraop_threads(int32_t n_threads);

/**
 * Get the syntaxdot version.
 *
 * The returned string must not be deallocated.
 */
char const *syntaxdot_version();

/**
 * Free a <tt>ByteBuffer</tt>.
 * @param buf The buffer to free.
 */
void syntaxdot_free_bytebuffer(ByteBuffer buf);

/**
 * Free a string allocated through this library.
 * @param s The string to free.
 */
void syntaxdot_free_string(char *s);

#ifdef __cplusplus
}
#endif

#endif //SYNTAXDOT_H
