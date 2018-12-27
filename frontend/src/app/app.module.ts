import { BrowserModule } from '@angular/platform-browser';
import { NgModule } from '@angular/core';

import { AppRoutingModule } from './app-routing.module';
import { AppComponent } from './app.component';
import { HomeComponent } from './home/home.component';
import { LaunchesComponent } from './launches/launches.component';
import { HttpClientModule } from '@angular/common/http'
import { AgmCoreModule }  from '@agm/core';

@NgModule({
  declarations: [
    AppComponent,
    HomeComponent,
    LaunchesComponent
  ],
  imports: [
    BrowserModule,
    AppRoutingModule,
    HttpClientModule,
    AgmCoreModule.forRoot({
      apiKey: 'AIzaSyCPHoYYa0dxDujbOnssEvPIAXwdQPuGsqA'
    }),
  ],
  providers: [],
  bootstrap: [AppComponent]
})
export class AppModule { }
