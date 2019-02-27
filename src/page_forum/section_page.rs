use sapper::{
    Request, 
    Response, 
    Result as SapperResult, 
    Error as SapperError, 
    Module as SapperModule,
    Router as SapperRouter};
use sapper_std::*;
use uuid::Uuid;
use crate::serde_json;

use crate::db;
// introduce macros
use sapper_std::res_html;
use crate::{
    AppWebContext,
    AppUser
};

use crate::dataservice::section::{
    Section,
    SectionNew,
    SectionEdit,
    UpdateSectionWeight
};
use crate::constants::NUMBER_ARTICLE_PER_PAGE;

use crate::middleware::permission_need_be_admin;


pub struct SectionPage;

impl SectionPage {

    pub fn section_create_page(req: &mut Request) -> SapperResult<Response> {
        let mut web = ext_type_owned!(req, AppWebContext).unwrap();

        res_html!("forum/new_section.html", web)
    }

    pub fn section_edit_page(req: &mut Request) -> SapperResult<Response> {
        let mut web = ext_type_owned!(req, AppWebContext).unwrap();
        let params = get_query_params!(req);
        let section_id = t_param_parse!(params, "id", Uuid);

        let section = Section::get_by_id(section_id).unwrap();

        web.add("section", &section);

        res_html!("forum/edit_section.html", web)
    }
    
    pub fn section_detail_page(req: &mut Request) -> SapperResult<Response> {
        let mut web = ext_type_owned!(req, AppWebContext).unwrap();
        let params = get_query_params!(req);
        
        let (path, _) = req.uri();
        let section_id = if path == "/blog_with_author" {
            let author_id = t_param_parse!(params, "author_id", Uuid);
            let section = Section::get_by_suser(author_id);
            if section.is_err() {
                return res_400!("no this section");
            }
            let section = section.unwrap();
            section.id
        }
        else {
            t_param_parse!(params, "id", Uuid)
        };

        let current_page = t_param_parse_default!(params, "current_page", i64, 1);

        let section_result = Section::get_by_id(section_id);
        if section_result.is_err() {
            return res_404!("Not Found");
        }
        
        let section = section_result.unwrap();
        let mut is_a_blog = false;
        if section.stype == 1 {
            is_a_blog = true;
        }
        let mut is_myown_blog = false;
        let mut is_admin = false;
        let mut is_login = false;
        match ext_type!(req, AppUser) {
            Some(user) => {
                if section.suser == Some(user.id) {
                    is_myown_blog = true;
                }
                if user.role >= 9 {
                    is_admin = true;
                }

                is_login = true;
                web.add("is_login", &is_login);
                web.add("user", &user);
            },
            None => {}
        }

        let total_item = Section::get_articles_count_belong_to_this(section.id);
        let total_page = (total_item / NUMBER_ARTICLE_PER_PAGE) as i64 + 1;

        let articles = Section::get_articles_paging_belong_to_this(section.id, current_page);

        web.add("section", &section);
        web.add("is_a_blog", &is_a_blog);
        web.add("is_myown_blog", &is_myown_blog);
        web.add("is_admin", &is_admin);
        web.add("total_item", &total_item);
        web.add("total_page", &total_page);
        web.add("current_page", &current_page);
        web.add("articles", &articles);

        res_html!("forum/section.html", web)
    }



    pub fn section_create(req: &mut Request) -> SapperResult<Response> {
        let params = get_form_params!(req);
        let title = t_param!(params, "title").to_owned();
        let description = t_param!(params, "description").to_owned();

        let section_new = SectionNew {
            title,
            description
        };

        match section_new.create() {
            Ok(section) => {
                res_redirect!(format!("/section?id={}", section.id))
            },
            Err(_) => {
                res_500!("section create error.")
            }
        }  
    }

    pub fn section_edit(req: &mut Request) -> SapperResult<Response> {
        let params = get_form_params!(req);
        let id = t_param_parse!(params, "id", Uuid);
        let title = t_param!(params, "title").to_owned();
        let description = t_param!(params, "description").to_owned();

        let section_edit = SectionEdit {
            id,
            title,
            description
        };

        match section_edit.update() {
            Ok(section) => {
                res_redirect!(format!("/section?id={}", section.id))
            },
            Err(_) => {
                res_500!("section edit error.")
            }
        }  
    }

    pub fn section_rearrange_page(req: &mut Request) -> SapperResult<Response> {
        let mut web = ext_type_owned!(req, AppWebContext).unwrap();

        let sections = Section::forum_sections();

        web.add("sections", &sections);

        res_html!("forum/arrange_sections.html", web)
    }


    pub fn section_rearrange(req: &mut Request) -> SapperResult<Response> {
        let mut web = ext_type_owned!(req, AppWebContext).unwrap();
        let params = get_form_params!(req);
        println!("{:?}", params);
        let order = t_arr_param!(params, "order");

        // print order
        println!("==> order {:?}", order);
        let sections = Section::forum_sections();
        for (i, section) in sections.iter().enumerate() {
            let update_section_weight = UpdateSectionWeight {
                id: section.id,
                weight: order[i].parse::<f64>().unwrap()
            };
            update_section_weight.update().unwrap();
        }
        
        res_redirect!("/p/section/rearrange")
    }

}


impl SapperModule for SectionPage {
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        match permission_need_be_admin(req) {
            Ok(_) => {
                // pass, nothing need to do here
            },
            Err(info) => {
                return Err(SapperError::Custom("No permission.".to_string()));
            }
        }
        
        Ok(())
    }

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.get("/section", Self::section_detail_page);
        router.get("/blog", Self::section_detail_page);
        router.get("/blog_with_author", Self::section_detail_page);

        router.get("/p/section/create", Self::section_create_page);
        router.get("/p/section/edit", Self::section_edit_page);
        router.post("/s/section/create", Self::section_create);
        router.post("/s/section/edit", Self::section_edit);

        router.get("/p/section/rearrange", Self::section_rearrange_page);
        router.post("/s/section/rearrange", Self::section_rearrange);


        Ok(())
    }
}


