// Format with vertical alignment.

use std::cmp;

use itertools::Itertools;
use rustc_ast::ast;
use rustc_span::{BytePos, Span};

use crate::comment::combine_strs_with_missing_comments;
use crate::config::ElementSpacing;
use crate::config::lists::*;
use crate::expr::rewrite_field;
use crate::items::{rewrite_struct_field, rewrite_struct_field_prefix};
use crate::lists::{
    ListFormatting, ListItem, Separator, apply_item_spacing, definitive_tactic, itemize_list,
    write_list,
};
use crate::rewrite::{Rewrite, RewriteContext, RewriteResult};
use crate::shape::{Indent, Shape};
use crate::source_map::SpanUtils;
use crate::spanned::Spanned;
use crate::utils::{
    contains_skip, is_attributes_extendable, mk_sp, rewrite_ident, trimmed_last_line_width,
};

pub(crate) trait AlignedItem {
    fn skip(&self) -> bool;
    fn has_attrs(&self) -> bool;
    fn get_span(&self) -> Span;
    fn rewrite_prefix(&self, context: &RewriteContext<'_>, shape: Shape) -> RewriteResult;
    fn rewrite_aligned_item(
        &self,
        context: &RewriteContext<'_>,
        shape: Shape,
        prefix_max_width: usize,
    ) -> RewriteResult;
}

impl AlignedItem for ast::FieldDef {
    fn skip(&self) -> bool {
        contains_skip(&self.attrs)
    }

    fn has_attrs(&self) -> bool {
        !self.attrs.is_empty()
    }

    fn get_span(&self) -> Span {
        self.span()
    }

    fn rewrite_prefix(&self, context: &RewriteContext<'_>, shape: Shape) -> RewriteResult {
        let attrs_str = self.attrs.rewrite_result(context, shape)?;
        let missing_span = if self.attrs.is_empty() {
            mk_sp(self.span.lo(), self.span.lo())
        } else {
            mk_sp(self.attrs.last().unwrap().span.hi(), self.span.lo())
        };
        let attrs_extendable = self.ident.is_none() && is_attributes_extendable(&attrs_str);
        let field_str = rewrite_struct_field_prefix(context, self)?;
        combine_strs_with_missing_comments(
            context,
            &attrs_str,
            &field_str,
            missing_span,
            shape,
            attrs_extendable,
        )
    }

    fn rewrite_aligned_item(
        &self,
        context: &RewriteContext<'_>,
        shape: Shape,
        prefix_max_width: usize,
    ) -> RewriteResult {
        rewrite_struct_field(context, self, shape, prefix_max_width)
    }
}

impl AlignedItem for ast::ExprField {
    fn skip(&self) -> bool {
        contains_skip(&self.attrs)
    }

    fn has_attrs(&self) -> bool {
        !self.attrs.is_empty()
    }

    fn get_span(&self) -> Span {
        self.span()
    }

    fn rewrite_prefix(&self, context: &RewriteContext<'_>, shape: Shape) -> RewriteResult {
        let attrs_str = self.attrs.rewrite_result(context, shape)?;
        let name = rewrite_ident(context, self.ident);
        let missing_span = if self.attrs.is_empty() {
            mk_sp(self.span.lo(), self.span.lo())
        } else {
            mk_sp(self.attrs.last().unwrap().span.hi(), self.span.lo())
        };
        combine_strs_with_missing_comments(
            context,
            &attrs_str,
            name,
            missing_span,
            shape,
            is_attributes_extendable(&attrs_str),
        )
    }

    fn rewrite_aligned_item(
        &self,
        context: &RewriteContext<'_>,
        shape: Shape,
        prefix_max_width: usize,
    ) -> RewriteResult {
        rewrite_field(context, self, shape, prefix_max_width)
    }
}

/// Rewrites `fields`, vertically aligning them when `struct_field_align_threshold` is set.
///
/// `spacing` controls the blank lines between fields. When fields are aligned, blank lines
/// delimit the alignment groups, so `spacing` also determines how the fields are grouped.
pub(crate) fn rewrite_with_alignment<T: AlignedItem>(
    fields: &[T],
    context: &RewriteContext<'_>,
    shape: Shape,
    span: Span,
    one_line_width: usize,
    spacing: ElementSpacing,
) -> Option<String> {
    let (spaces, group_index) = if context.config.struct_field_align_threshold() > 0 {
        group_aligned_items(context, fields, shape, spacing)
    } else {
        ("", fields.len() - 1)
    };
    let init = &fields[0..=group_index];
    let rest = &fields[group_index + 1..];
    let init_last_pos = if rest.is_empty() {
        span.hi()
    } else {
        // Decide whether the missing comments should stick to init or rest.
        let init_hi = init[init.len() - 1].get_span().hi();
        let rest_lo = rest[0].get_span().lo();
        let missing_span = mk_sp(init_hi, rest_lo);
        let missing_span = mk_sp(
            context.snippet_provider.span_after(missing_span, ","),
            missing_span.hi(),
        );

        let snippet = context.snippet(missing_span);
        if snippet.trim_start().starts_with("//") {
            let offset = snippet.lines().next().map_or(0, str::len);
            // 2 = "," + "\n"
            init_hi + BytePos(offset as u32 + 2)
        } else if snippet.trim_start().starts_with("/*") {
            let comment_lines = snippet
                .lines()
                .position(|line| line.trim_end().ends_with("*/"))
                .unwrap_or(0);

            let offset = snippet
                .lines()
                .take(comment_lines + 1)
                .collect::<Vec<_>>()
                .join("\n")
                .len();

            init_hi + BytePos(offset as u32 + 2)
        } else {
            missing_span.lo()
        }
    };
    let init_span = mk_sp(span.lo(), init_last_pos);
    let one_line_width = if rest.is_empty() { one_line_width } else { 0 };

    // if another group follows, we must force a separator
    let force_separator = !rest.is_empty();

    let result = rewrite_aligned_items_inner(
        context,
        init,
        init_span,
        shape.indent,
        one_line_width,
        force_separator,
        spacing,
    )?;
    if rest.is_empty() {
        Some(result + spaces)
    } else {
        let rest_span = mk_sp(init_last_pos, span.hi());
        let rest_str =
            rewrite_with_alignment(rest, context, shape, rest_span, one_line_width, spacing)?;
        Some(format!(
            "{}{}\n{}{}",
            result,
            spaces,
            &shape.indent.to_string(context.config),
            &rest_str
        ))
    }
}

fn struct_field_prefix_max_min_width<T: AlignedItem>(
    context: &RewriteContext<'_>,
    fields: &[T],
    shape: Shape,
) -> (usize, usize) {
    fields
        .iter()
        .map(|field| {
            field
                .rewrite_prefix(context, shape)
                .map(|field_str| trimmed_last_line_width(&field_str))
        })
        .fold_ok((0, usize::MAX), |(max_len, min_len), len| {
            (cmp::max(max_len, len), cmp::min(min_len, len))
        })
        .unwrap_or((0, 0))
}

fn rewrite_aligned_items_inner<T: AlignedItem>(
    context: &RewriteContext<'_>,
    fields: &[T],
    span: Span,
    offset: Indent,
    one_line_width: usize,
    force_trailing_separator: bool,
    spacing: ElementSpacing,
) -> Option<String> {
    // 1 = ","
    let item_shape = Shape::indented(offset, context.config).sub_width_opt(1)?;
    let (mut field_prefix_max_width, field_prefix_min_width) =
        struct_field_prefix_max_min_width(context, fields, item_shape);
    let max_diff = field_prefix_max_width.saturating_sub(field_prefix_min_width);
    if max_diff > context.config.struct_field_align_threshold() {
        field_prefix_max_width = 0;
    }

    let mut items = itemize_list(
        context.snippet_provider,
        fields.iter(),
        "}",
        ",",
        |field| field.get_span().lo(),
        |field| field.get_span().hi(),
        |field| field.rewrite_aligned_item(context, item_shape, field_prefix_max_width),
        span.lo(),
        span.hi(),
        false,
    )
    .collect::<Vec<_>>();
    apply_item_spacing(&mut items, spacing, |i| fields[i].has_attrs());

    let tactic = definitive_tactic(
        &items,
        ListTactic::HorizontalVertical,
        Separator::Comma,
        one_line_width,
    );

    if tactic == DefinitiveListTactic::Horizontal {
        // since the items fits on a line, there is no need to align them
        let do_rewrite =
            |field: &T| -> RewriteResult { field.rewrite_aligned_item(context, item_shape, 0) };
        fields
            .iter()
            .zip(items.iter_mut())
            .for_each(|(field, list_item): (&T, &mut ListItem)| {
                if list_item.item.is_ok() {
                    list_item.item = do_rewrite(field);
                }
            });
    }

    let separator_tactic = if force_trailing_separator {
        SeparatorTactic::Always
    } else {
        context.config.trailing_comma()
    };

    let fmt = ListFormatting::new(item_shape, context.config)
        .tactic(tactic)
        .trailing_separator(separator_tactic)
        .preserve_newline(true);
    write_list(&items, &fmt).ok()
}

/// Returns the index in `fields` up to which a field belongs to the current group.
/// The returned string is the group separator to use when rewriting the fields.
/// Groups are defined by the blank lines between fields, as determined by `spacing`.
fn group_aligned_items<T: AlignedItem>(
    context: &RewriteContext<'_>,
    fields: &[T],
    shape: Shape,
    spacing: ElementSpacing,
) -> (&'static str, usize) {
    let spans_multiple_lines = |field: &T| {
        field
            .rewrite_aligned_item(context, shape, 0)
            .map_or(false, |s| s.contains('\n'))
    };
    let mut index = 0;
    for i in 0..fields.len() - 1 {
        if fields[i].skip() {
            return ("", index);
        }
        let span = mk_sp(fields[i].get_span().hi(), fields[i + 1].get_span().lo());
        let snippet = context
            .snippet(span)
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join("\n");
        let mut own_lines = snippet.lines().dropping_back(1);
        let separated = match spacing {
            ElementSpacing::Preserve => own_lines.any(|l| l.trim().is_empty()),
            ElementSpacing::Never => false,
            ElementSpacing::Always => true,
            ElementSpacing::AroundAnnotated => {
                own_lines.any(|l| {
                    let l = l.trim();
                    l.is_empty() || l.starts_with("//") || l.starts_with("/*")
                }) || fields[i].has_attrs()
                    || fields[i + 1].has_attrs()
            }
            ElementSpacing::AroundMultiline => {
                // A comment on its own line makes the next field span multiple lines.
                own_lines.any(|l| {
                    let l = l.trim();
                    l.is_empty() || l.starts_with("//") || l.starts_with("/*")
                }) || spans_multiple_lines(&fields[i])
                    || spans_multiple_lines(&fields[i + 1])
            }
        };
        if separated {
            return ("\n", index);
        }
        index += 1;
    }
    ("", index)
}
