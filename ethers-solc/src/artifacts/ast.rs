//! Bindings for solc's `ast` output field

use crate::artifacts::serde_helpers;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::BTreeMap, fmt, fmt::Write, str::FromStr};

/// Represents the AST field in the solc output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ast {
    #[serde(rename = "absolutePath")]
    pub absolute_path: String,
    pub id: usize,
    #[serde(default, rename = "exportedSymbols")]
    pub exported_symbols: BTreeMap<String, Vec<usize>>,
    #[serde(rename = "nodeType")]
    pub node_type: NodeType,
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<Node>,

    /// Node attributes that were not deserialized.
    #[serde(flatten)]
    pub other: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    /// The node ID.
    pub id: usize,

    /// The node type.
    #[serde(rename = "nodeType")]
    pub node_type: NodeType,

    /// The location of the node in the source file.
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,

    /// Child nodes for some node types.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<Node>,

    /// Body node for some node types.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<Box<Node>>,

    /// Node attributes that were not deserialized.
    #[serde(flatten)]
    pub other: BTreeMap<String, serde_json::Value>,
}

impl Node {
    /// Deserialize a serialized node attribute.
    pub fn attribute<D: DeserializeOwned>(&self, key: impl AsRef<str>) -> Option<D> {
        // TODO: Can we avoid this clone?
        self.other.get(key.as_ref()).and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

/// Represents the source location of a node: `<start byte>:<length>:<source index>`.
///
/// The `length` and `index` can be -1 which is represented as `None`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub start: usize,
    pub length: Option<usize>,
    pub index: Option<usize>,
}

impl FromStr for SourceLocation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid_location = move || format!("{} invalid source location", s);

        let mut split = s.split(':');
        let start = split
            .next()
            .ok_or_else(invalid_location)?
            .parse::<usize>()
            .map_err(|_| invalid_location())?;
        let length = split
            .next()
            .ok_or_else(invalid_location)?
            .parse::<isize>()
            .map_err(|_| invalid_location())?;
        let index = split
            .next()
            .ok_or_else(invalid_location)?
            .parse::<isize>()
            .map_err(|_| invalid_location())?;

        let length = if length < 0 { None } else { Some(length as usize) };
        let index = if index < 0 { None } else { Some(index as usize) };

        Ok(Self { start, length, index })
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.start.fmt(f)?;
        f.write_char(':')?;
        if let Some(length) = self.length {
            length.fmt(f)?;
        } else {
            f.write_str("-1")?;
        }
        f.write_char(':')?;
        if let Some(index) = self.index {
            index.fmt(f)?;
        } else {
            f.write_str("-1")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    // Expressions
    Assignment,
    BinaryOperation,
    Conditional,
    ElementaryTypeNameExpression,
    FunctionCall,
    FunctionCallOptions,
    Identifier,
    IndexAccess,
    IndexRangeAccess,
    Literal,
    MemberAccess,
    NewExpression,
    TupleExpression,
    UnaryOperation,

    // Statements
    Block,
    Break,
    Continue,
    DoWhileStatement,
    EmitStatement,
    ExpressionStatement,
    ForStatement,
    IfStatement,
    InlineAssembly,
    PlaceholderStatement,
    Return,
    RevertStatement,
    TryStatement,
    UncheckedBlock,
    VariableDeclarationStatement,
    VariableDeclaration,
    WhileStatement,

    // Yul statements
    YulAssignment,
    YulBlock,
    YulBreak,
    YulContinue,
    YulExpressionStatement,
    YulLeave,
    YulForLoop,
    YulFunctionDefinition,
    YulIf,
    YulSwitch,
    YulVariableDeclaration,

    // Yul expressions
    YulFunctionCall,
    YulIdentifier,
    YulLiteral,

    // Yul literals
    YulLiteralValue,
    YulHexValue,

    // Definitions
    ContractDefinition,
    FunctionDefinition,
    EventDefinition,
    ErrorDefinition,
    ModifierDefinition,
    StructDefinition,
    EnumDefinition,
    UserDefinedValueTypeDefinition,

    // Directives
    PragmaDirective,
    ImportDirective,
    UsingForDirective,

    // Misc
    SourceUnit,
    InheritanceSpecifier,
    ElementaryTypeName,
    FunctionTypeName,
    ParameterList,
    TryCatchClause,
    ModifierInvocation,

    /// An unknown AST node type.
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_ast() {
        let ast = include_str!("../../test-data/ast/ast-erc4626.json");
        let _ast: Ast = serde_json::from_str(ast).unwrap();
    }
}
