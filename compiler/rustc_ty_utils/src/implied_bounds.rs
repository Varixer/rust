use rustc_hir::{def::DefKind, def_id::DefId};
use rustc_middle::query::Providers;
use rustc_middle::ty::{self, Ty, TyCtxt};

pub fn provide(providers: &mut Providers) {
    *providers = Providers { assumed_wf_types, ..*providers };
}

fn assumed_wf_types(tcx: TyCtxt<'_>, def_id: DefId) -> &ty::List<Ty<'_>> {
    match tcx.def_kind(def_id) {
        DefKind::Fn => {
            let sig = tcx.fn_sig(def_id).subst_identity();
            let liberated_sig = tcx.liberate_late_bound_regions(def_id, sig);
            liberated_sig.inputs_and_output
        }
        DefKind::AssocFn => {
            let sig = tcx.fn_sig(def_id).subst_identity();
            let liberated_sig = tcx.liberate_late_bound_regions(def_id, sig);
            let mut assumed_wf_types: Vec<_> =
                tcx.assumed_wf_types(tcx.parent(def_id)).as_slice().into();
            assumed_wf_types.extend(liberated_sig.inputs_and_output);
            tcx.mk_type_list(&assumed_wf_types)
        }
        DefKind::Impl { .. } => {
            match tcx.impl_trait_ref(def_id) {
                Some(trait_ref) => {
                    let types: Vec<_> = trait_ref.skip_binder().substs.types().collect();
                    tcx.mk_type_list(&types)
                }
                // Only the impl self type
                None => tcx.mk_type_list(&[tcx.type_of(def_id).subst_identity()]),
            }
        }
        DefKind::AssocConst | DefKind::AssocTy => tcx.assumed_wf_types(tcx.parent(def_id)),
        DefKind::OpaqueTy => match tcx.def_kind(tcx.parent(def_id)) {
            DefKind::TyAlias => ty::List::empty(),
            DefKind::AssocTy => tcx.assumed_wf_types(tcx.parent(def_id)),
            // Nested opaque types only occur in associated types:
            // ` type Opaque<T> = impl Trait<&'static T, AssocTy = impl Nested>; `
            // assumed_wf_types should include those of `Opaque<T>`, `Opaque<T>` itself
            // and `&'static T`.
            DefKind::OpaqueTy => bug!("unimplemented implied bounds for neseted opaque types"),
            def_kind @ _ => {
                bug!("unimplemented implied bounds for opaque types with parent {def_kind:?}")
            }
        },
        DefKind::Mod
        | DefKind::Struct
        | DefKind::Union
        | DefKind::Enum
        | DefKind::Variant
        | DefKind::Trait
        | DefKind::TyAlias
        | DefKind::ForeignTy
        | DefKind::TraitAlias
        | DefKind::TyParam
        | DefKind::Const
        | DefKind::ConstParam
        | DefKind::Static(_)
        | DefKind::Ctor(_, _)
        | DefKind::Macro(_)
        | DefKind::ExternCrate
        | DefKind::Use
        | DefKind::ForeignMod
        | DefKind::AnonConst
        | DefKind::InlineConst
        | DefKind::ImplTraitPlaceholder
        | DefKind::Field
        | DefKind::LifetimeParam
        | DefKind::GlobalAsm
        | DefKind::Closure
        | DefKind::Generator => ty::List::empty(),
    }
}
