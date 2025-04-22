// use anyhow::{Result, anyhow};
// use sqlparser::ast::{
//     BinaryOperator, Expr, Ident, Query as SqlQuery, SelectItem, Statement, Value,
// };
// use sqlparser::dialect::GenericDialect;
// use sqlparser::parser::Parser;
// use tantivy::query::{AllQuery, BooleanQuery, Occur, Query, RangeQuery, TermQuery};
// use tantivy::schema::{Field, IndexRecordOption, Schema, Term};

// pub fn parse_sql_statement(sql: &str) -> Result<Box<SqlQuery>> {
//     let dialect = GenericDialect {};
//     let ast = Parser::parse_sql(&dialect, sql)?;
//     let stmt = ast.into_iter().next().ok_or(anyhow!("Empty SQL"))?;

//     match stmt {
//         Statement::Query(query_box) => Ok(query_box),
//         _ => Err(anyhow!("Only SELECT statements are supported")),
//     }
// }

// /// Строит Tantivy Query из WHERE-условия SQL-запроса
// pub fn extract_filter_query(query: &SqlQuery, schema: &Schema) -> Result<Box<dyn Query>> {
//     let select = query
//         .body
//         .as_select()
//         .ok_or_else(|| anyhow!("Only SELECT statements are supported"))?;

//     match &select.selection {
//         Some(expr) => expr_to_query(expr, schema),
//         None => Ok(Box::new(AllQuery)),
//     }
// }

// /// Извлекает список полей из SELECT части SQL-запроса
// pub fn extract_projection(query: &SqlQuery, schema: &Schema) -> Result<Vec<Field>> {
//     let select = query
//         .body
//         .as_select()
//         .ok_or_else(|| anyhow!("Only SELECT statements are supported"))?;

//     let fields = select
//         .projection
//         .iter()
//         .map(|item| {
//             // 1) Получили Ident или сразу Err:
//             let ident = match item {
//                 SelectItem::UnnamedExpr(Expr::Identifier(ident)) => ident,
//                 other => return Err(anyhow!("Unsupported projection item: {:?}", other)),
//             };

//             // 2) Конвертируем Option (schema.get_field) в Result:
//             let field = schema
//                 .get_field(&ident.value)
//                 .map_err(|e| anyhow!(e.to_string()))?;

//             // 3) Проверяем, stored ли оно, или возвращаем Err:
//             if !schema.get_field_entry(field).is_stored() {
//                 return Err(anyhow!(
//                     "Field '{}' is not stored",
//                     schema.get_field_entry(field).name()
//                 ));
//             }

//             // 4) Всё ок → вернули field
//             Ok(field)
//         })
//         .collect::<Result<Vec<_>>>()?;

//     Ok(fields)
// }

// fn invert_operator(op: &BinaryOperator) -> Result<BinaryOperator> {
//     use BinaryOperator::*;
//     Ok(match op {
//         Gt => Lt,
//         GtEq => LtEq,
//         Lt => Gt,
//         LtEq => GtEq,
//         Eq => Eq,
//         NotEq => NotEq,
//         And => And,
//         Or => Or,
//         op => return Err(anyhow!("Unsupported operator: {}", op)),
//     })
// }

// fn build_leaf_query(
//     ident: &Ident,
//     val: &Value,
//     op: &BinaryOperator,
//     schema: &Schema,
// ) -> Result<Box<dyn Query>> {
//     let field_name = &ident.value;
//     let field = schema.get_field(field_name)?;

//     // Сначала определяем тип значения и создаём Term
//     let (term, _): (Term, &'static str) = match val {
//         Value::SingleQuotedString(s) => (Term::from_field_text(field, s), "text"),
//         Value::Boolean(b) => (Term::from_field_bool(field, *b), "bool"),
//         Value::Number(n, _) => {
//             if let Ok(parsed) = n.parse::<i64>() {
//                 (Term::from_field_i64(field, parsed), "i64")
//             } else if let Ok(parsed) = n.parse::<f64>() {
//                 (Term::from_field_f64(field, parsed), "f64")
//             } else {
//                 return Err(anyhow!("Invalid numeric literal"));
//             }
//         }
//         _ => {
//             return Err(anyhow!("Unsupported value type"));
//         }
//     };

//     // Теперь матчим по оператору
//     use BinaryOperator::*;
//     match op {
//         Eq => Ok(Box::new(TermQuery::new(term, IndexRecordOption::Basic))),
//         NotEq => Ok(Box::new(BooleanQuery::new(vec![(
//             Occur::MustNot,
//             Box::new(TermQuery::new(term, IndexRecordOption::Basic)),
//         )]))),

//         Gt | GtEq | Lt | LtEq => {
//             // Используем RangeQuery::new с Bound<Term>
//             use std::ops::Bound::*;
//             let (lower, upper) = match op {
//                 Gt => (Excluded(term), Unbounded),
//                 GtEq => (Included(term), Unbounded),
//                 Lt => (Unbounded, Excluded(term)),
//                 LtEq => (Unbounded, Included(term)),
//                 _ => unreachable!(),
//             };
//             Ok(Box::new(RangeQuery::new(lower, upper)))
//         }

//         _ => Err(anyhow!(format!("Unsupported operator: {:?}", op))),
//     }
// }

// fn expr_to_query(expr: &Expr, schema: &Schema) -> Result<Box<dyn Query>> {
//     match expr {
//         // Бинарная операция
//         Expr::BinaryOp { left, op, right } => {
//             match (&**left, &**right) {
//                 // Поле = Значение  => TermQuery
//                 (Expr::Identifier(ident), Expr::Value(vws)) => {
//                     build_leaf_query(ident, &vws.value, op, schema)
//                 }

//                 // Значение = Поле  => тоже TermQuery (переставим местами)
//                 (Expr::Value(vws), Expr::Identifier(ident)) => {
//                     let inverted_op = invert_operator(op)
//                         .map_err(|err| tantivy::TantivyError::InvalidArgument(err.to_string()))?;
//                     build_leaf_query(ident, &vws.value, &inverted_op, schema)
//                 }

//                 // Вложенные выражения: (a = 1) AND (b = 2)
//                 _ => {
//                     let left_query = expr_to_query(left, schema)?;
//                     let right_query = expr_to_query(right, schema)?;

//                     let query: Box<dyn Query> = match op {
//                         BinaryOperator::And => {
//                             Box::new(BooleanQuery::intersection(vec![left_query, right_query]))
//                         }
//                         BinaryOperator::Or => {
//                             Box::new(BooleanQuery::union(vec![left_query, right_query]))
//                         }
//                         _ => {
//                             return Err(anyhow!("Unsupported binary op: {:?}", op));
//                         }
//                     };

//                     Ok(query)
//                 }
//             }
//         }

//         // Просто отдельное поле — ошибка, оно ничего не означает само по себе
//         Expr::Identifier(_) | Expr::Value(_) => Err(anyhow!("Unpaired identifier or value")),

//         _ => Err(anyhow!("Unsupported expression")),
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use tantivy::schema::{FAST, STRING, Schema};

//     #[test]
//     fn test_sql_where_to_tantivy_query() -> anyhow::Result<()> {
//         // 1. Схема с двумя полями
//         let mut builder = Schema::builder();
//         builder.add_text_field("brand", STRING);
//         builder.add_text_field("category", STRING);
//         let schema = builder.build();

//         // 2. Пример SQL-запроса
//         let sql = "SELECT brand FROM products WHERE brand = 'Apple' AND category = 'Phones'";

//         // 3. Парсим в Tantivy Query
//         let sql_query = parse_sql_statement(sql)?;
//         let query = extract_filter_query(&sql_query, &schema)?;

//         // 4. Проверка, что вернулся Query с 2 условиями
//         let debug_str = format!("{:#?}", query);

//         assert_eq!(
//             debug_str.matches("TermQuery").count(),
//             2,
//             "Expected 2 TermQuery instances"
//         );
//         Ok(())
//     }

//     #[test]
//     fn test_sql_comparison_operators() -> anyhow::Result<()> {
//         // Создаём схему с нужными полями
//         let mut builder = Schema::builder();
//         builder.add_text_field("brand", STRING);
//         builder.add_f64_field("price", FAST);
//         builder.add_i64_field("rating", FAST);
//         let schema = builder.build();

//         // SQL-запрос с разными операторами
//         let sql = "
//                 SELECT brand, price FROM products
//                 WHERE brand = 'Apple'
//                   AND price > 100
//                   AND price <= 500
//                   AND rating != 3
//                   AND rating >= 1
//                   AND rating < 5
//             ";

//         // Парсим SQL → Tantivy Query
//         let sql_query = parse_sql_statement(sql)?;
//         let query = extract_filter_query(&sql_query, &schema)?;

//         //Проверяем, что это BooleanQuery с 6 условиями
//         let debug_str = format!("{:#?}", query);
//         assert_eq!(debug_str.matches("TermQuery").count(), 2);
//         assert_eq!(debug_str.matches("RangeQuery").count(), 4);

//         Ok(())
//     }
// }
