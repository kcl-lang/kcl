use anyhow;
use std::collections::HashMap;
use crate::{
    util::{build_word_index, parse_param_and_compile, Param},
    state::{LanguageServerSnapshot, Task, log_message},
    from_lsp::{self, file_path_from_url, kcl_pos},
    goto_def::{goto_definition, find_def,},
};
use lsp_types;
use crossbeam_channel::Sender;
use kclvm_config::modfile::get_pkg_root;
use kclvm_ast::ast::Stmt;


pub(crate) fn find_references (
    snapshot: LanguageServerSnapshot,
    params: lsp_types::ReferenceParams,
    sender: Sender<Task>,
) -> anyhow::Result<Option<Vec<lsp_types::Location>>> {
    // 1. find definition of current token
    let file = file_path_from_url(&params.text_document_position.text_document.uri)?;
    let path = from_lsp::abs_path(&params.text_document_position.text_document.uri)?;
    let db = snapshot.get_db(&path.clone().into())?;
    let pos = kcl_pos(&file, params.text_document_position.position);

    if let Some(def_resp) = goto_definition(&db.prog, &pos, &db.scope) {
        match def_resp {
            lsp_types::GotoDefinitionResponse::Scalar(def_loc) => {
                // get the def location
                if let Some(def_name) = match db.prog.pos_to_stmt(&pos) {
                    Some(node) => match node.node {
                        Stmt::Import(_) => None,
                        _ => match find_def(node.clone(), &pos, &db.scope) {
                            Some(def) => Some(def.get_name()),
                            None => None,
                        },
                    },
                    None => None,
                } {
                    // 2. find all occurrence of current token
                    // todo: decide the scope by the workspace root and the kcl.mod both, use the narrower scope
                    if let Some(root) = get_pkg_root(path.display().to_string().as_str()) {
                        match build_word_index(root) {
                            Ok(word_index) => {
                                return find_refs(def_loc, def_name, word_index);
                            },
                            Err(_) => {
                                let _ = log_message("build word index failed".to_string(), &sender);
                                return anyhow::Ok(None);
                            }
                        }
                    } else {
                        return Ok(None)
                    }
                }
            },
            _=> return Ok(None),
        }
    } else {
        log_message("Definition item not found, result in no reference".to_string(), &sender)?;
    }
    
    return Ok(None)
}

pub(crate) fn find_refs(def_loc:lsp_types::Location, name: String, word_index: HashMap<String, Vec<lsp_types::Location>>) 
-> anyhow::Result<Option<Vec<lsp_types::Location>>>{
    if let Some(locs) = word_index.get(name.as_str()).cloned() {
        return anyhow::Ok(Some(locs.into_iter().filter(|ref_loc|{
            // from location to real def
            // return if the real def location matches the def_loc
            let file_path = ref_loc.uri.path().to_string();
            match parse_param_and_compile(
                Param {
                    file: file_path.clone(),
                },
                None,
            ) {
                Ok((prog, scope, _)) => {
                    let ref_pos = kcl_pos(&file_path, ref_loc.range.start);
                    // find def from the ref_pos
                    if let Some(real_def) = goto_definition(&prog, &ref_pos, &scope) {
                        match real_def {
                            lsp_types::GotoDefinitionResponse::Scalar(real_def_loc) => {
                                real_def_loc == def_loc
                            },
                            _ => false
                        }
                    } else {
                        false
                    }

                }
                Err(_) => {
                    // todo log compilation error
                    return false;
                },
            }
        }).collect()));
    } else {
        return Ok(None)
    }
    
}

#[cfg(test)]
mod tests {
    //todo
    // todo assert
    #[test]
    fn test_find_refs() {
        
    }


}