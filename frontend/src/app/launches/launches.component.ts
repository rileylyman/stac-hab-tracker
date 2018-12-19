import { Component, OnInit } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { HabdataService, HABUpdate } from '../habdata.service';

@Component({
  selector: 'app-launches',
  templateUrl: './launches.component.html',
  styleUrls: ['./launches.component.scss']
})
export class LaunchesComponent implements OnInit {

  trip$: number;
  updates$: HABUpdate[];

  constructor(private route: ActivatedRoute, private data: HabdataService) { 
    this.route.params.subscribe( 
      params => this.trip$ = params.trip
    );
  }

  ngOnInit() {
    //this.data.getUpdates(this.trip$).subscribe(
    //  data => this.updates$ = data
    //);
    this.updates$ = [];
  }

}
