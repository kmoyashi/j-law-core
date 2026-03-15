//go:build cgo && !((darwin && amd64) || (darwin && arm64) || (linux && amd64) || (linux && arm64))

package jlawcore

/*
#error "j-law-go ships prebuilt native archives only for darwin/amd64, darwin/arm64, linux/amd64, and linux/arm64"
*/
import "C"
