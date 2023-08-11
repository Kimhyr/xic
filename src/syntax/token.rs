/*

identifier-token ::= identifier: "[a-zA-Z_][a-zA-Z_0-9]*"

integer-token ::= ['+'|'-'][0-9]*

decimal-token ::= integer-token ['.' integer-token]

string-token ::= '"' TODO '"'

boolean-token ::= 'true' | 'false'

literal-token ::= integer-token
                | decimal-token
                | string-token
                | boolean-token
                | 'null'

nonterminal-token ::= identifier-token
                    | literal-token

keyword-token ::= 'module' | 'type'
                | 'trait' | 'function'
                | 'value'

delimiterized-token ::= '{' | '}'
                      | '(' | ')'
                      | '<' | '>'
                      | '[' | ']'

punctuator-token ::=

token ::= keyword-token
        | delimiterized-token
        | nonterminal-token

module MyModule;

trait MyTrait<TypeType>
{
    alias Type = TypeType;

    function do_something(Type) -> Type;
}

type BitField: bit32 = bit1: bit
                     , bit2: bit
                     , bits3t5: (bit * 3)
                     , bits6t8: bit3
                     , bits: (bit * (32 - offset_of!(bits)));

type MyType<TypeType> = field1: (int, int)                              # Tuple
                      , field2: (first: TypeType, second: TypeType)     # Tagged Tuple
                      , field3: (Monday | Tuesday | Thursday)           # Enumerare
                      , field4: (int * 32)                              # Array
                      , field5: (int32 + int64)                         # Union
                      , field6: @bit8,                                  # Pointer
                      , field7: (() -> TypeType)                        # Function
                      , field8: (Identifier: String | Number: float64); # Algebraic

use core::ObjectType;

derive!(Debuggable)
extend MyType<TypeType>
    : ObjectType
{
    function create() -> Self =
        Self {};

    function destroy() -> Self = {};

    function do_something(@?self) -> Self
    {
        self.field1 = (21, 14);
        return Self = self.copy();
    }
}

*/

use crate::diagnostics::Position;

#[derive(Debug)]
pub struct Token {
    pub r#type: TokenType,
    position: Position,
}

impl Token {
    pub fn new(position: Position) -> Self {
        Self {
            r#type: TokenType::None,
            position,
        }
    }

    pub fn r#type(&self) -> &TokenType {
        &self.r#type
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    None,

    Identifier(String),

    // Literals
    Bits(u64),
    Integer(i64),
    Decimal(f64),
    Boolean(bool),
    String(String),
    Character(char),
    BitsLiteral,
    IntegerLiteral,
    DecimalLiteral,

    //
    // Words
    //

    // Determiner Words
    Module,   // 'module'
    Trait,    // 'trait'
    Type,     // 'type'
    Extend,   // 'extend'
    Function, // 'function'
    Value,    // 'value'

    // Actional Words
    Use,    // 'use'
    Return, // 'return'

    // Valuable Words
    True,
    False,

    //
    // Types
    //

    // Bit Types
    Bit,   // 'bit'
    Bit8,  // 'bit8'
    Bit16, // 'bit16'
    Bit32, // 'bit32'
    Bit64, // 'bit64'

    // Integer Types
    Int,   // 'int'
    Int8,  // 'int8'
    Int16, // 'int16'
    Int32, // 'int32'
    Int64, // 'int64'

    // Floating-point Types
    Float,   // 'float'
    Float8,  // 'float8'
    Float16, // 'float16'
    Float32, // 'float32'
    Float64, // 'float64'

    // Logical Types
    Bool, // 'bool'

    // Textual Types
    Char,   // 'char'
    Char8,  // 'char8'
    Char16, // 'char16'
    Char32, // 'char32'

    //
    // Punctuators
    //

    // Regular Punctuators
    Apostrophe,      // '''
    QutationMark,    // '"'
    FullStop,        // '.'
    Comma,           // ','
    Colon,           // ':'
    Semicolon,       // ';'
    EqualsSign,      // '='
    PlusSign,        // '+'
    MinuxSign,       // '-'
    Asterisk,        // '*'
    Solidus,         // '/'
    ReverseSolidus,  // '\'
    VerticalLine,    // '|'
    ExclamationMark, // '!'
    QuestionMark,    // '?'
    ComercialAt,     // '@'
    NumberSign,      // '#'

    RightwardsArrow, // '->'

    // Delimiterized Punctuators
    LeftCurlyBracket,   // '{'
    RightCurlyBracket,  // '}'
    LeftParenthesis,    // '('
    RightParenthesis,   // ')'
    LeftAngleBracket,   // '<'
    RightAngleBracket,  // '>'
    LeftSquareBracket,  // '['
    RightSquareBracket, // ']'
}
