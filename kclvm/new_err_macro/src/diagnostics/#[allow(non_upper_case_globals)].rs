#[allow(non_upper_case_globals)] 

const
_DERIVE_DiagnosticBuilder_FOR_ThisIsAnErr1 : () =
{
    impl DiagnosticBuilder for ThisIsAnErr1
        {
            fn into_diagnostic(self,) -> Diagnostic
                {            
                    ThisIsAnErr1 { pos : ref __binding_0, } =>
                        {
                            {
                                let codectx_pendant = CodeCtxPendant ::new(self.stringify! (pos),.clone()) ; 
                                let codectx_sentence = Sentence :: new_sentence_str(Box :: new(codectx_pendant), Message :: Str(\"oh no! this is an error!\".to_string()),);
                                                                                                                            }
                                                                                                                                    } let title_pendant = HeaderPendant ::
                                                                                                                                                new(\"error\", \"E0124\".to_string()) ; let title_sentence = Sentence            ::
                                                                                                                                                            new_sentence_str(Box :: new(title_pendant), Message ::
                                                                                                                                                                        Str(\"oh no! this is an error!\".to_string()),) ; let label_pendant
                                                                                                                                                                                    = LabelPendant :: new(\"error\".to_string()) ; let label_sentence =
                                                                                                                                                                                                Sentence ::
                                                                                                                                                                                                            new_sentence_str(Box :: new(label_pendant), Message ::
                                                                                                                                                                                                            Str(\"oh no! this is an error!\".to_string()),) ; ; let mut
                                                                                                                                                                                                            diagnostic = Diagnostic :: new() ;
                                                                                                                                                                                                            diagnostic.add_sentence(title_sentence) ;
                                                                                                                                                                                                            diagnostic.add_sentence(codectx_sentence) ;
                                                                                                                                                                                                            diagnostic.add_sentence(label_sentence) ;
                                                                                                                                                                                                            diagnostic.add_sentence(nopendant_sentence) ; diagnostic
                                                                                                                                                                                                    }
                                                                                                                                                                                                }
                                                                                                                                                                                            } ;