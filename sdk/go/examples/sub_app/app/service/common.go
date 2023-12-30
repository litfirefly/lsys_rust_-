package service

import (
	"github.com/gin-gonic/gin"
	lSysApi "lsysrest/lsysrest"
	"net/http"
)

var restApi *lSysApi.RestApi

func init() {
	if restApi == nil {
		restApi = lSysApi.NewRestApi(&lSysApi.RestApiConfig{
			//应用在 https://www.lsys.cc/app.html#/user/app 申请
			AppId:          "1212f",                            //应用ID
			AppSecret:      "3f95638a1e07b87df2b64e09c2541dac", //应用Secret
			AppHost:        "http://www.lsys.cc",               //应用HOST
			AppOAuthSecret: "2a97bf1b4f075b0ca7467e7c6b223f89", //应用OauthSecret
			AppOAuthHost:   "http://www.lsys.cc/oauth.html",
		})
	}
}
func GetRestApi() *lSysApi.RestApi {
	return restApi
}

func ErrorPage(ctx *gin.Context, msg string) {
	ctx.HTML(http.StatusOK, "err.html", gin.H{
		"msg": msg,
	})
}
