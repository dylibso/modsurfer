<?php
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# source: proto/v1/api.proto

/**
 * The language (or most similar match) used to produce a wasm module.
 *
 * Protobuf type <code>SourceLanguage</code>
 */
class SourceLanguage
{
    /**
     * Generated from protobuf enum <code>Unknown = 0;</code>
     */
    const Unknown = 0;
    /**
     * Generated from protobuf enum <code>Rust = 1;</code>
     */
    const Rust = 1;
    /**
     * Generated from protobuf enum <code>Go = 2;</code>
     */
    const Go = 2;
    /**
     * Generated from protobuf enum <code>C = 3;</code>
     */
    const C = 3;
    /**
     * Generated from protobuf enum <code>Cpp = 4;</code>
     */
    const Cpp = 4;
    /**
     * Generated from protobuf enum <code>AssemblyScript = 5;</code>
     */
    const AssemblyScript = 5;
    /**
     * Generated from protobuf enum <code>Swift = 6;</code>
     */
    const Swift = 6;
    /**
     * Generated from protobuf enum <code>JavaScript = 7;</code>
     */
    const JavaScript = 7;
    /**
     * Generated from protobuf enum <code>Haskell = 8;</code>
     */
    const Haskell = 8;
    /**
     * Generated from protobuf enum <code>Zig = 9;</code>
     */
    const Zig = 9;

    private static $valueToName = [
        self::Unknown => 'Unknown',
        self::Rust => 'Rust',
        self::Go => 'Go',
        self::C => 'C',
        self::Cpp => 'Cpp',
        self::AssemblyScript => 'AssemblyScript',
        self::Swift => 'Swift',
        self::JavaScript => 'JavaScript',
        self::Haskell => 'Haskell',
        self::Zig => 'Zig',
    ];

    public static function name($value)
    {
        if (!isset(self::$valueToName[$value])) {
            throw new UnexpectedValueException(sprintf(
                    'Enum %s has no name defined for value %s', __CLASS__, $value));
        }
        return self::$valueToName[$value];
    }


    public static function value($name)
    {
        $const = __CLASS__ . '::' . strtoupper($name);
        if (!defined($const)) {
            throw new UnexpectedValueException(sprintf(
                    'Enum %s has no value defined for name %s', __CLASS__, $name));
        }
        return constant($const);
    }
}
