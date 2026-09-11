package markstone

import (
	"errors"
	"fmt"
)

var (
	// ErrNullArgument is returned when a null argument is passed to a native function.
	ErrNullArgument = errors.New("markstone: null argument passed to native function")

	// ErrInvalidUTF8 is returned when the input byte sequence is not valid UTF-8.
	ErrInvalidUTF8 = errors.New("markstone: input contains invalid UTF-8")

	// ErrInputTooLarge is returned when the input length exceeds 4 MiB.
	ErrInputTooLarge = errors.New("markstone: input size exceeds 4 MiB limit")

	// ErrDepthExceeded is returned when block nesting depth exceeds 64.
	ErrDepthExceeded = errors.New("markstone: block nesting depth exceeds limit of 64")

	// ErrInternal is returned when an internal error or caught panic occurs in the core.
	ErrInternal = errors.New("markstone: internal error or caught panic")
)

func statusToError(status int32) error {
	switch status {
	case 0:
		return nil
	case 1:
		return ErrNullArgument
	case 2:
		return ErrInvalidUTF8
	case 3:
		return ErrInputTooLarge
	case 4:
		return ErrDepthExceeded
	case 5:
		return ErrInternal
	default:
		return fmt.Errorf("markstone: unknown error status %d", status)
	}
}
